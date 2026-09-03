document
  .querySelector(`nav a[href="${location.pathname}"]`)
  ?.setAttribute('aria-current', 'page');
