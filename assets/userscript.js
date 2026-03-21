// ==UserScript==
// @name            Analog Library
// @description     Redirect the ACM Digital Library to an Analog Library interface
// @author          apropos
// @version         1.0
// @include         *://dl.acm.org/*
// @run-at          document-start
// ==/UserScript==

// This is a userscript. It requires a userscript manager, such as ViolentMonkey.
window.location.replace("__HOST__" + window.location.pathname);
