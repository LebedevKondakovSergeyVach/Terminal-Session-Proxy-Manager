const storageKey = 'starlight-theme';
const normalize = (value) => value === 'light' || value === 'dark' ? value : 'auto';

export function initializeThemeSwitch() {
	if (customElements.get('starlight-theme-switch')) return;
	const root = document.documentElement;
	const systemScheme = matchMedia('(prefers-color-scheme: light)');
	const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
	const readStored = () => {
		try {
			return normalize(localStorage.getItem(storageKey));
		} catch {
			return undefined;
		}
	};
	let preference = readStored() ?? 'auto';
	let memoryOnly = false;
	let revision = 0;
	let activeTransition;

	const resolvedTheme = () => preference === 'auto'
		? (systemScheme.matches ? 'light' : 'dark')
		: preference;
	const syncButtons = () => {
		document.querySelectorAll('starlight-theme-switch button').forEach((button) => {
			button.setAttribute('aria-pressed', String(button.dataset.themeValue === preference));
		});
	};
	const applyTheme = () => {
		root.dataset.theme = resolvedTheme();
		syncButtons();
	};
	const stopTransition = () => {
		revision++;
		const previous = activeTransition;
		activeTransition = undefined;
		previous?.skipTransition();
		delete root.dataset.themeTransition;
		for (const name of ['x', 'y', 'radius']) root.style.removeProperty(`--tspm-theme-${name}`);
	};
	const restore = () => {
		stopTransition();
		// A failed write must not let a stale stored preference override the
		// user's choice on this document. Other documents keep their own state.
		if (!memoryOnly) preference = readStored() ?? preference;
		applyTheme();
	};
	const choose = (value, event, button) => {
		stopTransition();
		preference = normalize(value);
		try {
			if (preference === 'auto') localStorage.removeItem(storageKey);
			else localStorage.setItem(storageKey, preference);
			memoryOnly = false;
		} catch {
			memoryOnly = true;
		}
		syncButtons();
		if (!document.startViewTransition || reducedMotion.matches || document.hidden || root.dataset.theme === resolvedTheme()) {
			applyTheme();
			return;
		}

		const rect = button.getBoundingClientRect();
		const x = event.detail === 0 ? rect.left + rect.width / 2 : event.clientX;
		const y = event.detail === 0 ? rect.top + rect.height / 2 : event.clientY;
		const radius = Math.hypot(Math.max(x, innerWidth - x), Math.max(y, innerHeight - y));
		const currentRevision = revision;
		root.style.setProperty('--tspm-theme-x', `${x}px`);
		root.style.setProperty('--tspm-theme-y', `${y}px`);
		root.style.setProperty('--tspm-theme-radius', `${radius}px`);
		root.dataset.themeTransition = '';
		let transition;
		try {
			transition = document.startViewTransition(() => {
				if (revision === currentRevision) applyTheme();
			});
		} catch {
			stopTransition();
			applyTheme();
			return;
		}
		activeTransition = transition;
		transition.ready.catch(() => {
			// CSS owns the reveal timing, including browsers without WAAPI
			// pseudo-element animation support. Skipped transitions still apply.
			if (activeTransition === transition) transition.skipTransition();
		});
		const finish = () => {
			if (activeTransition !== transition) return;
			activeTransition = undefined;
			delete root.dataset.themeTransition;
			for (const name of ['x', 'y', 'radius']) root.style.removeProperty(`--tspm-theme-${name}`);
			applyTheme();
		};
		transition.finished.then(finish, finish);
		transition.updateCallbackDone.catch(() => {
			if (activeTransition === transition) applyTheme();
		});
	};

	class StarlightThemeSwitch extends HTMLElement {
		connectedCallback() {
			syncButtons();
		}
	}
	customElements.define('starlight-theme-switch', StarlightThemeSwitch);
	applyTheme();
	document.addEventListener('click', (event) => {
		const button = event.target instanceof Element
			? event.target.closest('starlight-theme-switch button[data-theme-value]')
			: null;
		if (button) choose(button.dataset.themeValue, event, button);
	});
	systemScheme.addEventListener('change', () => {
		if (preference !== 'auto') return;
		stopTransition();
		applyTheme();
	});
	window.addEventListener('storage', (event) => {
		if (event.key !== storageKey && event.key !== null) return;
		stopTransition();
		preference = readStored() ?? preference;
		memoryOnly = false;
		applyTheme();
	});
	window.addEventListener('pageshow', (event) => {
		if (event.persisted) restore();
	});
	window.addEventListener('pagereveal', restore);
	const leave = () => {
		stopTransition();
		applyTheme();
	};
	window.addEventListener('pageswap', leave);
	window.addEventListener('pagehide', leave);
	document.addEventListener('visibilitychange', () => {
		if (document.hidden) leave();
	});
}
