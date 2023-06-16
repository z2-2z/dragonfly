use crate::{
    input::HasPacketVector,
    mutators::packet::PacketMutatorTuple,
};
use libafl::prelude::{
    Corpus,
    CorpusId,
    Error,
    HasCorpus,
    HasRand,
    HasSolutions,
    MOptMode,
    MutationResult,
    Mutator,
    Named,
    Rand,
    StdRand,
};
use std::marker::PhantomData;

const PERIOD_PILOT_COEF: f64 = 5000.0;
const V_MAX: f64 = 1.0;
const V_MIN: f64 = 0.05;

struct MOpt {
    /// Random number generator
    rand: StdRand,
    /// The number of total findings (unique crashes and unique interesting paths). This is equivalent to `state.corpus().count() + state.solutions().count()`;
    total_finds: usize,
    /// The number of finds before until last swarm.
    finds_until_last_swarm: usize,
    /// These w_* and g_* values are the coefficients for updating variables according to the PSO algorithms
    w_init: f64,
    /// These w_* and g_* values are the coefficients for updating variables according to the PSO algorithms
    w_end: f64,
    /// These w_* and g_* values are the coefficients for updating variables according to the PSO algorithms
    w_now: f64,
    /// These w_* and g_* values are the coefficients for updating variables according to the PSO algorithms
    g_now: f64,
    /// These w_* and g_* values are the coefficients for updating variables according to the PSO algorithms
    g_max: f64,
    /// The number of mutation operators
    operator_num: usize,
    /// The number of swarms that we want to employ during the pilot fuzzing mode
    swarm_num: usize,
    /// We'll generate inputs for `period_pilot` times before we call pso_update in pilot fuzzing module
    period_pilot: usize,
    /// We'll generate inputs for `period_core` times before we call pso_update in core fuzzing module
    period_core: usize,
    /// The number of testcases generated during this pilot fuzzing mode
    pilot_time: usize,
    /// The number of testcases generated during this core fuzzing mode
    core_time: usize,
    /// The swarm identifier that we are currently using in the pilot fuzzing mode
    swarm_now: usize,
    /// A parameter for the PSO algorithm
    x_now: Vec<Vec<f64>>,
    /// A parameter for the PSO algorithm
    l_best: Vec<Vec<f64>>,
    /// A parameter for the PSO algorithm
    eff_best: Vec<Vec<f64>>,
    /// A parameter for the PSO algorithm
    g_best: Vec<f64>,
    /// A parameter for the PSO algorithm
    v_now: Vec<Vec<f64>>,
    /// The probability that we want to use to choose the mutation operator.
    probability_now: Vec<Vec<f64>>,
    /// The fitness for each swarm, we'll calculate the fitness in the pilot fuzzing mode and use the best one in the core fuzzing mode
    swarm_fitness: Vec<f64>,
    /// (Pilot Mode) Finds by each operators. This vector is used in pso_update
    pilot_operator_finds: Vec<Vec<u64>>,
    /// (Pilot Mode) Finds by each operator till now.
    pilot_operator_finds_v2: Vec<Vec<u64>>,
    /// (Pilot Mode) The number of mutation operator used. This vector is used in pso_update
    pilot_operator_cycles: Vec<Vec<u64>>,
    /// (Pilot Mode) The number of mutation operator used till now
    pilot_operator_cycles_v2: Vec<Vec<u64>>,
    /// (Pilot Mode) The number of mutation operator used till last execution
    pilot_operator_cycles_v3: Vec<Vec<u64>>,
    /// Vector used in pso_update
    operator_finds_puppet: Vec<u64>,
    /// (Core Mode) Finds by each operators. This vector is used in pso_update
    core_operator_finds: Vec<u64>,
    /// (Core Mode) Finds by each operator till now.
    core_operator_finds_v2: Vec<u64>,
    /// (Core Mode) The number of mutation operator used. This vector is used in pso_update
    core_operator_cycles: Vec<u64>,
    /// (Core Mode) The number of mutation operator used till now
    core_operator_cycles_v2: Vec<u64>,
    /// (Core Mode) The number of mutation operator used till last execution
    core_operator_cycles_v3: Vec<u64>,
}

impl MOpt {
    fn new(operator_num: usize, swarm_num: usize, rand_seed: u64) -> Result<Self, Error> {
        let mut mopt = Self {
            rand: StdRand::with_seed(rand_seed),
            total_finds: 0,
            finds_until_last_swarm: 0,
            w_init: 0.9,
            w_end: 0.3,
            w_now: 0.0,
            g_now: 0.0,
            g_max: 5000.0,
            operator_num,
            swarm_num,
            period_pilot: 50000,
            period_core: 500000,
            pilot_time: 0,
            core_time: 0,
            swarm_now: 0,
            x_now: vec![vec![0.0; operator_num]; swarm_num],
            l_best: vec![vec![0.0; operator_num]; swarm_num],
            eff_best: vec![vec![0.0; operator_num]; swarm_num],
            g_best: vec![0.0; operator_num],
            v_now: vec![vec![0.0; operator_num]; swarm_num],
            probability_now: vec![vec![0.0; operator_num]; swarm_num],
            swarm_fitness: vec![0.0; swarm_num],
            pilot_operator_finds: vec![vec![0; operator_num]; swarm_num],
            pilot_operator_finds_v2: vec![vec![0; operator_num]; swarm_num],
            pilot_operator_cycles: vec![vec![0; operator_num]; swarm_num],
            pilot_operator_cycles_v2: vec![vec![0; operator_num]; swarm_num],
            pilot_operator_cycles_v3: vec![vec![0; operator_num]; swarm_num],
            operator_finds_puppet: vec![0; operator_num],
            core_operator_finds: vec![0; operator_num],
            core_operator_finds_v2: vec![0; operator_num],
            core_operator_cycles: vec![0; operator_num],
            core_operator_cycles_v2: vec![0; operator_num],
            core_operator_cycles_v3: vec![0; operator_num],
        };
        mopt.pso_initialize()?;
        Ok(mopt)
    }

    /// initialize pso
    #[allow(clippy::cast_precision_loss)]
    pub fn pso_initialize(&mut self) -> Result<(), Error> {
        if self.g_now > self.g_max {
            self.g_now = 0.0;
        }
        self.w_now = (self.w_init - self.w_end) * (self.g_max - self.g_now) / self.g_max + self.w_end;

        for swarm in 0..self.swarm_num {
            let mut total_x_now = 0.0;
            let mut x_sum = 0.0;
            for i in 0..self.operator_num {
                self.x_now[swarm][i] = (self.rand.below(7000) as f64) * 0.0001 + 0.1;
                total_x_now += self.x_now[swarm][i];
                self.v_now[swarm][i] = 0.1;
                self.l_best[swarm][i] = 0.5;
                self.g_best[i] = 0.5;
            }

            for i in 0..self.operator_num {
                self.x_now[swarm][i] /= total_x_now;
            }

            for i in 0..self.operator_num {
                self.v_now[swarm][i] = self.w_now * self.v_now[swarm][i]
                    + (self.rand.below(1000) as f64) * 0.001 * (self.l_best[swarm][i] - self.x_now[swarm][i])
                    + (self.rand.below(1000) as f64) * 0.001 * (self.g_best[i] - self.x_now[swarm][i]);
                self.x_now[swarm][i] += self.v_now[swarm][i];

                self.x_now[swarm][i] = self.x_now[swarm][i].clamp(V_MIN, V_MAX);

                x_sum += self.x_now[swarm][i];
            }

            for i in 0..self.operator_num {
                self.x_now[swarm][i] /= x_sum;
                if i == 0 {
                    self.probability_now[swarm][i] = self.x_now[swarm][i];
                } else {
                    self.probability_now[swarm][i] = self.probability_now[swarm][i - 1] + self.x_now[swarm][i];
                }
            }
            if self.probability_now[swarm][self.operator_num - 1] < 0.99 || self.probability_now[swarm][self.operator_num - 1] > 1.01 {
                return Err(Error::illegal_state("MOpt: Error in pso_update".to_string()));
            }
        }
        Ok(())
    }

    /// Update the `PSO` algorithm parameters
    /// See <https://github.com/puppet-meteor/MOpt-AFL/blob/master/MOpt/afl-fuzz.c#L10623>
    #[allow(clippy::cast_precision_loss)]
    pub fn pso_update(&mut self) -> Result<(), Error> {
        self.g_now += 1.0;
        if self.g_now > self.g_max {
            self.g_now = 0.0;
        }
        self.w_now = ((self.w_init - self.w_end) * (self.g_max - self.g_now) / self.g_max) + self.w_end;

        let mut operator_finds_sum = 0;

        for i in 0..self.operator_num {
            self.operator_finds_puppet[i] = self.core_operator_finds[i];

            for j in 0..self.swarm_num {
                self.operator_finds_puppet[i] += self.pilot_operator_finds[j][i];
            }
            operator_finds_sum += self.operator_finds_puppet[i];
        }

        for i in 0..self.operator_num {
            if self.operator_finds_puppet[i] > 0 {
                self.g_best[i] = (self.operator_finds_puppet[i] as f64) / (operator_finds_sum as f64);
            }
        }

        for swarm in 0..self.swarm_num {
            let mut x_sum = 0.0;
            for i in 0..self.operator_num {
                self.probability_now[swarm][i] = 0.0;
                self.v_now[swarm][i] = self.w_now * self.v_now[swarm][i]
                    + (self.rand.below(1000) as f64) * 0.001 * (self.l_best[swarm][i] - self.x_now[swarm][i])
                    + (self.rand.below(1000) as f64) * 0.001 * (self.g_best[i] - self.x_now[swarm][i]);
                self.x_now[swarm][i] += self.v_now[swarm][i];

                self.x_now[swarm][i] = self.x_now[swarm][i].clamp(V_MIN, V_MAX);

                x_sum += self.x_now[swarm][i];
            }

            for i in 0..self.operator_num {
                self.x_now[swarm][i] /= x_sum;
                if i == 0 {
                    self.probability_now[swarm][i] = self.x_now[swarm][i];
                } else {
                    self.probability_now[swarm][i] = self.probability_now[swarm][i - 1] + self.x_now[swarm][i];
                }
            }
            if self.probability_now[swarm][self.operator_num - 1] < 0.99 || self.probability_now[swarm][self.operator_num - 1] > 1.01 {
                return Err(Error::illegal_state("MOpt: Error in pso_update".to_string()));
            }
        }
        self.swarm_now = 0;

        // After pso_update, go back to pilot-fuzzing module
        Ok(())
    }

    /// This function is used to decide the operator that we want to apply next
    /// see <https://github.com/puppet-meteor/MOpt-AFL/blob/master/MOpt/afl-fuzz.c#L397>
    #[allow(clippy::cast_precision_loss)]
    pub fn select_algorithm(&mut self) -> Result<usize, Error> {
        let mut res = 0;
        let mut sentry = 0;

        let operator_num = self.operator_num;

        // Fetch a random sele value
        let select_prob: f64 = self.probability_now[self.swarm_now][operator_num - 1] * ((self.rand.below(10000) as f64) * 0.0001);

        for i in 0..operator_num {
            if i == 0 {
                if select_prob < self.probability_now[self.swarm_now][i] {
                    res = i;
                    break;
                }
            } else if select_prob < self.probability_now[self.swarm_now][i] {
                res = i;
                sentry = 1;
                break;
            }
        }

        if (sentry == 1 && select_prob < self.probability_now[self.swarm_now][res - 1]) || (res + 1 < operator_num && select_prob > self.probability_now[self.swarm_now][res + 1]) {
            return Err(Error::illegal_state("MOpt: Error in select_algorithm".to_string()));
        }
        Ok(res)
    }
}

pub struct MOptPacketMutator<I, P, S, M>
where
    M: PacketMutatorTuple<P, S>,
    S: HasRand,
{
    mopt: MOpt,
    mode: MOptMode,
    finds_before: usize,
    mutators: M,
    max_stack_pow: u64,
    phantom: PhantomData<(I, P, S)>,
}

impl<I, P, S, M> MOptPacketMutator<I, P, S, M>
where
    M: PacketMutatorTuple<P, S>,
    S: HasRand,
{
    pub fn new(state: &mut S, mutators: M, max_stack_pow: u64, swarm_num: usize) -> Result<Self, Error> {
        let seed = state.rand_mut().next();
        let mopt = MOpt::new(mutators.len(), swarm_num, seed)?;

        Ok(Self {
            mopt,
            mode: MOptMode::Pilotfuzzing,
            finds_before: 0,
            mutators,
            max_stack_pow,
            phantom: PhantomData,
        })
    }
}

impl<I, P, S, M> Mutator<I, S> for MOptPacketMutator<I, P, S, M>
where
    I: HasPacketVector<Packet = P>,
    M: PacketMutatorTuple<P, S>,
    S: HasRand + HasCorpus + HasSolutions,
{
    fn mutate(&mut self, state: &mut S, input: &mut I, stage_idx: i32) -> Result<MutationResult, Error> {
        self.finds_before = state.corpus().count() + state.solutions().count();
        let idx = self.schedule_packet(state, input);
        let packet = &mut input.packets_mut()[idx];
        self.scheduled_mutate(state, packet, stage_idx)
    }

    #[allow(clippy::cast_precision_loss)]
    fn post_exec(&mut self, state: &mut S, _stage_idx: i32, _corpus_idx: Option<CorpusId>) -> Result<(), Error> {
        let before = self.finds_before;
        let after = state.corpus().count() + state.solutions().count();

        let key_module = self.mode;
        match key_module {
            MOptMode::Corefuzzing => {
                self.mopt.core_time += 1;

                if after > before {
                    let diff = after - before;
                    self.mopt.total_finds += diff;
                    for i in 0..self.mopt.operator_num {
                        if self.mopt.core_operator_cycles_v2[i] > self.mopt.core_operator_cycles_v3[i] {
                            self.mopt.core_operator_finds_v2[i] += diff as u64;
                        }
                    }
                }

                if self.mopt.core_time > self.mopt.period_core {
                    self.mopt.core_time = 0;
                    let total_finds = self.mopt.total_finds;
                    self.mopt.finds_until_last_swarm = total_finds;
                    for i in 0..self.mopt.operator_num {
                        self.mopt.core_operator_finds[i] = self.mopt.core_operator_finds_v2[i];
                        self.mopt.core_operator_cycles[i] = self.mopt.core_operator_cycles_v2[i];
                    }
                    self.mopt.pso_update()?;
                    self.mode = MOptMode::Pilotfuzzing;
                }
            },
            MOptMode::Pilotfuzzing => {
                self.mopt.pilot_time += 1;
                let swarm_now = self.mopt.swarm_now;

                if after > before {
                    let diff = after - before;
                    self.mopt.total_finds += diff;
                    for i in 0..self.mopt.operator_num {
                        if self.mopt.pilot_operator_cycles_v2[swarm_now][i] > self.mopt.pilot_operator_cycles_v3[swarm_now][i] {
                            self.mopt.pilot_operator_finds_v2[swarm_now][i] += diff as u64;
                        }
                    }
                }

                #[allow(clippy::cast_lossless)]
                if self.mopt.pilot_time > self.mopt.period_pilot {
                    let new_finds = self.mopt.total_finds - self.mopt.finds_until_last_swarm;
                    let f = (new_finds as f64) / ((self.mopt.pilot_time as f64) / (PERIOD_PILOT_COEF));
                    self.mopt.swarm_fitness[swarm_now] = f;
                    self.mopt.pilot_time = 0;
                    let total_finds = self.mopt.total_finds;
                    self.mopt.finds_until_last_swarm = total_finds;

                    for i in 0..self.mopt.operator_num {
                        let mut eff = 0.0;
                        if self.mopt.pilot_operator_cycles_v2[swarm_now][i] > self.mopt.pilot_operator_cycles[swarm_now][i] {
                            eff = ((self.mopt.pilot_operator_finds_v2[swarm_now][i] - self.mopt.pilot_operator_finds[swarm_now][i]) as f64)
                                / ((self.mopt.pilot_operator_cycles_v2[swarm_now][i] - self.mopt.pilot_operator_cycles[swarm_now][i]) as f64);
                        }

                        if self.mopt.eff_best[swarm_now][i] < eff {
                            self.mopt.eff_best[swarm_now][i] = eff;
                            self.mopt.l_best[swarm_now][i] = self.mopt.x_now[swarm_now][i];
                        }

                        self.mopt.pilot_operator_finds[swarm_now][i] = self.mopt.pilot_operator_finds_v2[swarm_now][i];
                        self.mopt.pilot_operator_cycles[swarm_now][i] = self.mopt.pilot_operator_cycles_v2[swarm_now][i];
                    }

                    self.mopt.swarm_now += 1;

                    if self.mopt.swarm_num == 1 {
                        // If there's only 1 swarm, then no core_fuzzing mode.
                        self.mopt.pso_update()?;
                    } else if self.mopt.swarm_now == self.mopt.swarm_num {
                        self.mode = MOptMode::Corefuzzing;

                        for i in 0..self.mopt.operator_num {
                            self.mopt.core_operator_cycles_v2[i] = self.mopt.core_operator_cycles[i];
                            self.mopt.core_operator_cycles_v3[i] = self.mopt.core_operator_cycles[i];
                            self.mopt.core_operator_finds_v2[i] = self.mopt.core_operator_finds[i];
                        }

                        let mut swarm_eff = 0.0;
                        let mut best_swarm = 0;
                        for i in 0..self.mopt.swarm_num {
                            if self.mopt.swarm_fitness[i] > swarm_eff {
                                swarm_eff = self.mopt.swarm_fitness[i];
                                best_swarm = i;
                            }
                        }

                        self.mopt.swarm_now = best_swarm;
                    }
                }
            },
        }
        Ok(())
    }
}

impl<I, P, S, M> MOptPacketMutator<I, P, S, M>
where
    I: HasPacketVector<Packet = P>,
    M: PacketMutatorTuple<P, S>,
    S: HasRand,
{
    fn schedule_packet(&self, state: &mut S, input: &I) -> usize {
        state.rand_mut().below(input.packets().len() as u64) as usize
    }

    fn iterations(&self, state: &mut S) -> u64 {
        1 << (1 + state.rand_mut().below(self.max_stack_pow))
    }

    fn core_mutate(&mut self, state: &mut S, packet: &mut P, stage_idx: i32) -> Result<MutationResult, Error> {
        let mut result = MutationResult::Skipped;

        for i in 0..self.mopt.operator_num {
            self.mopt.core_operator_cycles_v3[i] = self.mopt.core_operator_cycles_v2[i];
        }

        for _ in 0..self.iterations(state) {
            let mutation = self.mopt.select_algorithm()?;
            let outcome = self.mutators.get_and_mutate(mutation, state, packet, stage_idx)?;

            #[cfg(test)]
            {
                if outcome == MutationResult::Mutated {
                    println!("Ran mutation #{}", mutation);
                }
            }

            if outcome == MutationResult::Mutated {
                result = MutationResult::Mutated;
            }

            self.mopt.core_operator_cycles_v2[mutation] += 1;
        }

        Ok(result)
    }

    fn pilot_mutate(&mut self, state: &mut S, packet: &mut P, stage_idx: i32) -> Result<MutationResult, Error> {
        let mut result = MutationResult::Skipped;
        let swarm_now;

        {
            swarm_now = self.mopt.swarm_now;

            for i in 0..self.mopt.operator_num {
                self.mopt.pilot_operator_cycles_v3[swarm_now][i] = self.mopt.pilot_operator_cycles_v2[swarm_now][i];
            }
        }

        for _ in 0..self.iterations(state) {
            let mutation = self.mopt.select_algorithm()?;
            let outcome = self.mutators.get_and_mutate(mutation, state, packet, stage_idx)?;

            #[cfg(test)]
            {
                if outcome == MutationResult::Mutated {
                    println!("Ran mutation #{}", mutation);
                }
            }

            if outcome == MutationResult::Mutated {
                result = MutationResult::Mutated;
            }

            self.mopt.pilot_operator_cycles_v2[swarm_now][mutation] += 1;
        }

        Ok(result)
    }

    fn scheduled_mutate(&mut self, state: &mut S, packet: &mut P, stage_idx: i32) -> Result<MutationResult, Error> {
        #[cfg(test)]
        println!("--- NEW MUTATION RUN ---");

        let mode = self.mode;
        match mode {
            MOptMode::Corefuzzing => self.core_mutate(state, packet, stage_idx),
            MOptMode::Pilotfuzzing => self.pilot_mutate(state, packet, stage_idx),
        }
    }
}

impl<I, P, S, M> Named for MOptPacketMutator<I, P, S, M>
where
    M: PacketMutatorTuple<P, S>,
    S: HasRand,
{
    fn name(&self) -> &str {
        "MOptPacketMutator"
    }
}
