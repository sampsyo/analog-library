// Creating a new bookmark with the following as the "URL" (and anything as the
// name) will redirect to the Analog Library interface when the bookmark is
// clicked while a paper in the ACM Digital Library is open + focused.

javascript:(function() { window.location.replace("__HOST__" + window.location.pathname) })()
