# TODO
* Convert `Result<_,&'static str>` to something `impl Error`
* Handle `io::Error`s more gracefully
* Make idempotent by making all changes at end (creation of intermediate directories, renaming of
  file)
* `restore` flag that allows you to move files back to their original destination
* print help info if no arg specified
* add man page
* Allow users to specify alternate .dumpster name/location via env var
