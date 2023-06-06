#!/bin/bash

cat << EOF
ServerName			"FuzzMe"
ServerType			standalone
Port				2121
ScoreboardFile off
PidFile /dev/null
ScoreboardScrub off
TransferLog none
SystemLog /dev/null
UseReverseDNS off
WtmpLog off
AllowOverwrite on

User				$USER
Group				$USER

AuthOrder mod_auth_file.c
AuthFileOptions InsecurePerms
AuthUserFile $(realpath ./passwd)
AuthGroupFile $(realpath ./group)

MaxInstances                    5
MaxStoreFileSize                1 Kb
MaxRetrieveFileSize             1 Kb
DeleteAbortedStores on

# Set the maximum number of seconds a data connection is allowed
# to "stall" before being aborted.
TimeoutStalled			60

UseSendfile off
Umask 0000
DefaultRoot /tmp/ftproot

<Anonymous /tmp/ftproot>
  RequireValidShell off
  AnonRequirePassword		off

  # Maximum clients with message
  MaxClients			2 "Sorry, max %m users -- try again later"

  User				ftp
  Group				ftp

  # Limit WRITE everywhere in the anonymous chroot
  <Limit WRITE>
    DenyAll
  </Limit>

  # An upload directory that allows storing files but not retrieving
  # or creating directories.
  <Directory uploads/*>
    <Limit READ>
      DenyAll
    </Limit>

    <Limit WRITE>
      AllowAll
    </Limit>
  </Directory>
</Anonymous>
EOF