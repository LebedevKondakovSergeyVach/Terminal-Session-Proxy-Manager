// A history restore keeps the outgoing MD3 state but does not rerun its
// DOMContentLoaded bootstrap. Reset before the incoming transition snapshot.
(() => {
	const reset = () => {
		document.documentElement.removeAttribute('data-md3-route-state');
		document.documentElement.removeAttribute('data-md3-sidebar-route-state');
		document.querySelectorAll('.md3-navigation-pending').forEach((link) => {
			link.classList.remove('md3-navigation-pending');
		});
	};
	window.addEventListener('pageshow', (event) => {
		if (event.persisted) reset();
	});
	const observeTransition = (event) => {
		// The outgoing document can also receive a skipped ready promise,
		// which otherwise rejects when that document is restored from BFCache.
		event.viewTransition?.ready.catch(() => {});
	};
	window.addEventListener('pagereveal', (event) => {
		observeTransition(event);
		reset();
	});
	window.addEventListener('pageswap', observeTransition);

	// MD3 delays navigation. A second click in that window must not queue a
	// second location change, especially when the first page enters BFCache.
	document.addEventListener('click', (event) => {
		if (document.documentElement.dataset.md3RouteState !== 'leaving') return;
		if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
		const link = event.target instanceof Element ? event.target.closest('a[href]') : null;
		if (!(link instanceof HTMLAnchorElement) || link.target || link.hasAttribute('download')) return;
		event.preventDefault();
	}, { capture: true });
})();
