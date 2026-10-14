import { readFileSync } from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import test from 'node:test';

const themeSource = readFileSync(new URL('../src/scripts/theme-switch.js', import.meta.url), 'utf8');
const recoverySource = readFileSync(new URL('../src/scripts/navigation-recovery.js', import.meta.url), 'utf8');
const scrollSource = readFileSync(new URL('../src/scripts/navigation-scroll.js', import.meta.url), 'utf8');
const bootstrapSource = readFileSync(new URL('../src/components/ThemeProvider.astro', import.meta.url), 'utf8')
	.match(/<script is:inline>([\s\S]*?)<\/script>/)[1];

class Events {
	listeners = new Map();
	addEventListener(name, listener) {
		const listeners = this.listeners.get(name) ?? [];
		listeners.push(listener);
		this.listeners.set(name, listeners);
	}
	emit(name, event = {}) {
		for (const listener of this.listeners.get(name) ?? []) listener(event);
	}
}
class Element {}
class Anchor extends Element {
	target = '';
	download = false;
	closest(selector) { return selector === 'a[href]' ? this : null; }
	hasAttribute() { return this.download; }
}
class Button extends Element {
	constructor(value) {
		super();
		this.dataset = { themeValue: value };
		this.attributes = {};
	}
	setAttribute(name, value) { this.attributes[name] = value; }
	closest(selector) { return selector.includes('starlight-theme-switch') ? this : null; }
	getBoundingClientRect() { return { left: 10, top: 20, width: 40, height: 40 }; }
}
const deferred = () => {
	let resolve, reject;
	const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
	return { promise, resolve, reject };
};
function setup({ stored = null, readFails = false, writeFails = false, transitions = false } = {}) {
	const window = new Events();
	const document = new Events();
	const system = Object.assign(new Events(), { matches: true });
	const motion = Object.assign(new Events(), { matches: false });
	const buttons = ['light', 'auto', 'dark', 'light', 'auto', 'dark'].map((value) => new Button(value));
	const animations = [];
	const properties = new Map();
	const root = {
		dataset: {},
		style: {
			setProperty: (name, value) => properties.set(name, value),
			getPropertyValue: (name) => properties.get(name) ?? '',
			removeProperty: (name) => properties.delete(name),
		},
		removeAttribute(name) {
			delete this.dataset[name.slice(5).replace(/-([a-z])/g, (_, c) => c.toUpperCase())];
		},
		animate(...args) { animations.push(args); },
	};
	document.documentElement = root;
	document.querySelectorAll = (selector) => selector === 'starlight-theme-switch button' ? buttons : [];
	const registry = new Map();
	const storage = {
		value: stored,
		getItem() { if (readFails) throw new Error('blocked read'); return this.value; },
		setItem(key, value) { if (writeFails) throw new Error('blocked write'); this.value = value; },
		removeItem() { if (writeFails) throw new Error('blocked write'); this.value = null; },
	};
	const pending = [];
	if (transitions) document.startViewTransition = (update) => {
		const ready = deferred(), finished = deferred(), updated = deferred();
		const transition = {
			ready: ready.promise, finished: finished.promise, updateCallbackDone: updated.promise,
			skipped: false,
			skipTransition() { this.skipped = true; },
		};
		pending.push({ transition, ready, finished, updated, update });
		return transition;
	};
	const context = vm.createContext({
		window, document, localStorage: storage, Element, HTMLElement: Element, HTMLAnchorElement: Anchor, innerWidth: 800, innerHeight: 600,
		matchMedia: (query) => query.includes('color-scheme') ? system : motion,
		customElements: { get: (name) => registry.get(name), define: (name, value) => registry.set(name, value) },
	});
	vm.runInContext(bootstrapSource, context);
	vm.runInContext(themeSource.replace('export function', 'function') + '\ninitializeThemeSwitch();', context);
	const click = (value, detail = 1) => document.emit('click', {
		target: buttons.find((button) => button.dataset.themeValue === value), detail, clientX: 100, clientY: 80,
	});
	return { context, window, document, root, system, motion, storage, pending, buttons, animations, click };
}

test('legacy and invalid stored preferences use Auto and all copies stay synchronized', () => {
	for (const stored of [null, '', 'auto', 'invalid']) {
		const env = setup({ stored });
		assert.equal(env.root.dataset.theme, 'light');
		assert.equal(env.buttons.filter((button) => button.attributes['aria-pressed'] === 'true').length, 2);
		env.system.matches = false;
		env.system.emit('change');
		assert.equal(env.root.dataset.theme, 'dark');
	}
});

test('blocked reads and failed writes preserve an explicit choice across system changes and history restore', () => {
	for (const options of [{ readFails: true, writeFails: true }, { writeFails: true }]) {
		const env = setup(options);
		env.click('dark');
		env.system.emit('change');
		env.window.emit('pageshow', { persisted: true });
		assert.equal(env.root.dataset.theme, 'dark');
		assert.equal(env.buttons[2].attributes['aria-pressed'], 'true');
		assert.equal(env.buttons[5].attributes['aria-pressed'], 'true');
	}
});

test('storage events and BFCache restores reapply another document\'s preference', () => {
	const env = setup({ stored: 'dark' });
	assert.equal(env.root.dataset.theme, 'dark');
	for (const name of ['storage', 'pagereveal', 'pageshow']) {
		env.storage.value = 'light';
		env.window.emit(name, { key: 'starlight-theme', persisted: true });
		assert.equal(env.root.dataset.theme, 'light');
		env.storage.value = 'dark';
		env.window.emit(name, { key: 'starlight-theme', persisted: true });
		assert.equal(env.root.dataset.theme, 'dark');
	}
});

test('rapid clicks cannot let a skipped callback or completion overwrite the newest transition', async () => {
	const env = setup({ transitions: true });
	const unhandled = [];
	const onRejection = (reason) => unhandled.push(reason);
	process.on('unhandledRejection', onRejection);
	try {
		env.click('dark');
		env.pending[0].update();
		env.click('light');
		assert.equal(env.pending[0].transition.skipped, true);
		env.pending[0].update();
		assert.equal(env.root.dataset.theme, 'dark');
		env.pending[0].ready.reject(new Error('skipped'));
		env.pending[0].finished.reject(new Error('interrupted'));
		env.pending[0].updated.reject(new Error('interrupted'));
		await new Promise((resolve) => setImmediate(resolve));
		assert.equal(env.root.dataset.themeTransition, '');
		env.pending[1].update();
		env.pending[1].ready.resolve();
		env.pending[1].updated.resolve();
		await new Promise((resolve) => setImmediate(resolve));
		assert.equal(env.root.dataset.theme, 'light');
		assert.equal(env.root.style.getPropertyValue('--tspm-theme-x'), '100px');
		env.pending[1].finished.resolve();
		await new Promise((resolve) => setImmediate(resolve));
		assert.equal(env.root.dataset.themeTransition, undefined);
		assert.deepEqual(unhandled, []);
	} finally {
		process.off('unhandledRejection', onRejection);
	}
});

test('keyboard reveal originates at the button and leaving cancels without stale markers', async () => {
	const env = setup({ transitions: true });
	assert.equal(env.document.listeners.get('click').length, 1);
	assert.equal(env.system.listeners.get('change').length, 1);
	env.click('dark', 0);
	env.pending[0].update();
	env.pending[0].ready.resolve();
	await new Promise((resolve) => setImmediate(resolve));
	assert.equal(env.root.style.getPropertyValue('--tspm-theme-x'), '30px');
	assert.equal(env.root.style.getPropertyValue('--tspm-theme-y'), '40px');
	env.window.emit('pagehide');
	assert.equal(env.pending[0].transition.skipped, true);
	assert.equal(env.root.dataset.themeTransition, undefined);
	assert.equal(env.root.dataset.theme, 'dark');
});

test('initial pageshow must not cancel a theme reveal started before assets finish loading', () => {
	const env = setup({ transitions: true });
	env.click('dark');
	env.window.emit('pageshow', { persisted: false });
	assert.equal(env.pending[0].transition.skipped, false);
	assert.equal(env.root.dataset.themeTransition, '');
});

test('reduced motion and unavailable transition APIs apply the theme immediately', () => {
	const env = setup({ transitions: true });
	env.motion.matches = true;
	env.click('dark');
	assert.equal(env.root.dataset.theme, 'dark');
	assert.equal(env.pending.length, 0);
	env.motion.matches = false;
	env.document.startViewTransition = () => { throw new Error('unavailable'); };
	env.click('light');
	assert.equal(env.root.dataset.theme, 'light');
	assert.equal(env.root.dataset.themeTransition, undefined);
});

test('history reveal clears MD3 outgoing route state and pending link markers', () => {
	const env = setup();
	let pending = true;
	env.document.querySelectorAll = (selector) => selector === '.md3-navigation-pending'
		? [{ classList: { remove: () => { pending = false; } } }]
		: env.buttons;
	vm.runInContext(recoverySource, env.context);
	for (const name of ['pagereveal', 'pageshow']) {
		env.root.dataset.md3RouteState = 'leaving';
		env.root.dataset.md3SidebarRouteState = 'entering';
		pending = true;
		env.window.emit(name, { persisted: true });
		assert.equal(env.root.dataset.md3RouteState, undefined);
		assert.equal(env.root.dataset.md3SidebarRouteState, undefined);
		assert.equal(pending, false);
	}
});

test('initial pageshow keeps navigation pending; repeat same-tab clicks are blocked but modifiers are preserved', () => {
	const env = setup();
	vm.runInContext(recoverySource, env.context);
	env.root.dataset.md3RouteState = 'leaving';
	env.window.emit('pageshow', { persisted: false });
	assert.equal(env.root.dataset.md3RouteState, 'leaving');
	let prevented = 0;
	const target = new Anchor();
	const click = (extra = {}) => env.document.emit('click', {
		target, button: 0, preventDefault: () => prevented++, ...extra,
	});
	click();
	assert.equal(prevented, 1);
	for (const flag of ['metaKey', 'ctrlKey', 'shiftKey', 'altKey']) click({ [flag]: true });
	click({ button: 1 });
	target.target = '_blank';
	click();
	target.target = '';
	target.download = true;
	click();
	assert.equal(prevented, 1);
});

test('native transition scroll compensation preserves history positions and tolerates skipped ready', async () => {
	const env = setup();
	const values = new Map();
	Object.assign(env.context, {
		sessionStorage: { getItem: (key) => values.get(key) ?? null, setItem: (key, value) => values.set(key, value) },
		scrollY: 640,
		location: { hash: '' },
		getComputedStyle: () => ({ animationName: '-ua-view-transition-group-anim-main', animationFillMode: 'both', animationTimingFunction: 'linear', animationDuration: '320ms' }),
	});
	vm.runInContext(scrollSource, env.context);
	env.window.emit('pageswap', { activation: { navigationType: 'traverse' } });
	env.context.scrollY = 160;
	const ready = deferred();
	env.window.emit('pagereveal', { viewTransition: { ready: ready.promise } });
	ready.resolve();
	await new Promise((resolve) => setImmediate(resolve));
	assert.equal(env.animations.length, 2);
	assert.deepEqual(Array.from(env.animations[0][0].top), ['0px', '-480px']);
	assert.deepEqual(Array.from(env.animations[1][0].top), ['480px', '0px']);
	assert.equal(env.animations[0][1].duration, 320);
	const skipped = deferred();
	env.window.emit('pagereveal', { viewTransition: { ready: skipped.promise } });
	skipped.reject(new Error('Transition was skipped'));
	await new Promise((resolve) => setImmediate(resolve));
	assert.equal(env.animations.length, 2);
	Object.defineProperty(env.context, 'sessionStorage', { get() { throw new Error('blocked'); } });
	env.window.emit('pageswap', {});
	env.window.emit('pagereveal', { viewTransition: { ready: Promise.resolve() } });
	await new Promise((resolve) => setImmediate(resolve));
	assert.equal(env.animations.length, 2);
});

test('both outgoing and incoming documents consume skipped native ready promises', async () => {
	const env = setup();
	vm.runInContext(recoverySource, env.context);
	for (const name of ['pageswap', 'pagereveal']) {
		const ready = deferred();
		env.window.emit(name, { viewTransition: { ready: ready.promise } });
		ready.reject(new Error('Transition was skipped'));
	}
	await new Promise((resolve) => setImmediate(resolve));
});
