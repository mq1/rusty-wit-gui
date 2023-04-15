#!/usr/bin/env osascript -l JavaScript
// from https://github.com/balena-io/etcher/blob/6fae328f1fdf88b8eda7b922f676424b8c3b0bb7/lib/shared/catalina-sudo/sudo-askpass.osascript-en.js

ObjC.import('stdlib')

const app = Application.currentApplication()
app.includeStandardAdditions = true

const result = app.displayDialog('rusty-wit-gui needs privileged access in order to format disks.\n\nType your password to allow this.', {
  defaultAnswer: '',
  withIcon: 'caution',
  buttons: ['Cancel', 'Ok'],
  defaultButton: 'Ok',
  hiddenAnswer: true,
})

if (result.buttonReturned === 'Ok') {
  result.textReturned
} else {
  $.exit(255)
}
