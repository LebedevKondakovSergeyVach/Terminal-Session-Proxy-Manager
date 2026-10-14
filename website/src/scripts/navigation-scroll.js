// The upstream cam-shaft helper awaits ready without handling cancellation.
// Keep its scroll compensation, with guarded storage and skipped transitions.
(() => {
	const scrollKey = 'vtbagCamshaftScrollY';
	const navigationKey = 'vtbagCamshaftNavigationType';
	const read = (key) => {
		try { return sessionStorage.getItem(key); } catch { return null; }
	};
	window.addEventListener('pageswap', (event) => {
		try {
			sessionStorage.setItem(scrollKey, String(scrollY));
			sessionStorage.setItem(navigationKey, event.activation?.navigationType ?? 'unknown');
		} catch {
			// Without storage, skip scroll compensation across documents.
		}
	});
	window.addEventListener('pagereveal', (event) => {
		const transition = event.viewTransition;
		if (!transition) return;
		const previousScroll = read(scrollKey);
		if (read(navigationKey) !== 'traverse' && location.hash) {
			try {
				document.getElementById(decodeURIComponent(location.hash.slice(1)))?.scrollIntoView({ behavior: 'instant', block: 'start' });
			} catch {
				// An invalid fragment must not interrupt navigation.
			}
		}
		const incomingScroll = scrollY;
		const outgoingScroll = Number.parseFloat(previousScroll ?? '');
		transition.ready.then(() => {
			if (!Number.isFinite(outgoingScroll)) return;
			const style = getComputedStyle(document.documentElement, '::view-transition-group(main)');
			if (style.animationName.replaceAll('"', '') !== '-ua-view-transition-group-anim-main') return;
			const duration = Number.parseFloat(style.animationDuration) * (style.animationDuration.endsWith('ms') ? 1 : 1000);
			const options = {
				composite: 'accumulate', fill: style.animationFillMode,
				easing: style.animationTimingFunction, duration,
			};
			document.documentElement.animate({ top: ['0px', `${incomingScroll - outgoingScroll}px`] }, {
				...options, pseudoElement: '::view-transition-old(main)',
			});
			document.documentElement.animate({ top: [`${outgoingScroll - incomingScroll}px`, '0px'] }, {
				...options, pseudoElement: '::view-transition-new(main)',
			});
		}).catch(() => {
			// ready rejects when a transition is skipped. Some browsers also
			// lack pseudo-element animations; native navigation still completes.
		});
	});
})();
