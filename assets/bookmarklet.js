(function () {
  let path = window.location.pathname;
  if (path.startsWith("/doi/")) {
    path = path.replace(/\/(abs|pdf|epdf|fullHtml)\//, "/");
    window.location.replace("http://__HOST__" + path);
  } else {
    console.error("this path does not look like a DOI");
  }
})();
