Output doc/demo.gif

Set FontSize 12
Set Width 1200
Set Height 600
Set Padding 20
Set WindowBar Colorful
Set Theme "Catppuccin Frappe"

# Setup
Hide
Type "cp ./target/release/sshh /usr/local/bin"
Enter
Type "cp ./samples/dummy.cfg ~/.ssh/config"
Enter
Type "export EDITOR=vim"
Enter
Type "clear"
Enter
Show

# Run tool
Type "sshh"
Sleep 2s
Enter
Sleep 1s

# Move
Down
Sleep 500ms
Down
Sleep 500ms
Up
Sleep 500ms
Sleep 1s

# Edit
Type e
Sleep 1s
Type "/foo2"
Sleep 500ms
Enter
Right
Type "xxxx"
Sleep 500ms
Type "i my-ssh-server"
Sleep 500ms
Escape
Type ":wq"
Enter

# See
Sleep 3s