type Point = { x: number; y: number };
type View = Point & { scale: number };

class DiagramViewer extends HTMLElement {
	connectedCallback() {
		const dialog = this.querySelector<HTMLDialogElement>('dialog')!;
		const stage = this.querySelector<HTMLElement>('.diagram-stage')!;
		const canvas = this.querySelector<HTMLElement>('.diagram-canvas')!;
		const title = this.querySelector<HTMLElement>('#diagram-title')!;
		const output = this.querySelector<HTMLOutputElement>('.diagram-scale')!;
		const template = this.querySelector<HTMLTemplateElement>('template')!;
		const zoomIn = this.querySelector<HTMLButtonElement>('[data-action="in"]')!;
		const zoomOut = this.querySelector<HTMLButtonElement>('[data-action="out"]')!;
		let source: HTMLPreElement | undefined;
		let svg: SVGSVGElement | undefined;
		let opener: HTMLButtonElement | undefined;
		let originalStyle: string | null = null;
		let originalOverflow = '';
		let originalScroll = { x: 0, y: 0 };
		let width = 1;
		let height = 1;
		let view: View = { x: 0, y: 0, scale: 1 };
		const pointers = new Map<number, Point>();
		let gesture: { view: View; midpoint: Point; distance: number } | undefined;

		const fitScale = () => Math.min(Math.max(1, stage.clientWidth - 24) / width, Math.max(1, stage.clientHeight - 24) / height, 1);
		const centre = () => ({ x: stage.clientWidth / 2, y: stage.clientHeight / 2 });
		const constrain = (position: number, size: number, available: number) =>
			size <= available ? (available - size) / 2 : Math.max(available - size, Math.min(0, position));
		const paint = (next: View) => {
			view = {
				scale: next.scale,
				x: constrain(next.x, width * next.scale, stage.clientWidth),
				y: constrain(next.y, height * next.scale, stage.clientHeight),
			};
			canvas.style.transform = `translate(${view.x}px, ${view.y}px) scale(${view.scale})`;
			canvas.dataset.scale = String(view.scale);
			canvas.dataset.x = String(view.x);
			canvas.dataset.y = String(view.y);
			const percent = `${Math.round(view.scale * 100)}%`;
			if (output.value !== percent) output.value = percent;
			zoomIn.disabled = view.scale >= 4;
			zoomOut.disabled = view.scale <= fitScale() + 0.001;
		};
		const zoom = (scale: number, at = centre(), start = view, offset = { x: 0, y: 0 }) => {
			const next = Math.max(fitScale(), Math.min(4, scale));
			paint({
				scale: next,
				x: at.x - (at.x - start.x) * next / start.scale + offset.x,
				y: at.y - (at.y - start.y) * next / start.scale + offset.y,
			});
		};
		const fit = () => paint({ x: 0, y: 0, scale: fitScale() });
		const actual = () => paint({ x: 0, y: 0, scale: 1 });
		const point = (event: PointerEvent | WheelEvent): Point => {
			const bounds = stage.getBoundingClientRect();
			return { x: event.clientX - bounds.left, y: event.clientY - bounds.top };
		};
		const geometry = () => {
			const [a, b = a] = [...pointers.values()];
			return { midpoint: { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 }, distance: Math.hypot(a.x - b.x, a.y - b.y) };
		};
		const startGesture = () => {
			gesture = pointers.size ? { view: { ...view }, ...geometry() } : undefined;
			stage.classList.toggle('is-panning', pointers.size > 0);
		};

		const open = (pre: HTMLPreElement, button: HTMLButtonElement) => {
			const diagram = pre.querySelector<SVGSVGElement>('svg');
			if (!diagram || dialog.open) return;
			source = pre;
			svg = diagram;
			opener = button;
			originalStyle = svg.getAttribute('style');
			originalScroll = { x: pre.scrollLeft, y: pre.scrollTop };
			width = svg.viewBox.baseVal.width;
			height = svg.viewBox.baseVal.height;
			title.textContent = svg.querySelector('title')?.textContent || button.dataset.title!;
			// Move the live SVG so its IDs, CSS and arrow references have one owner.
			pre.style.minHeight = `${pre.getBoundingClientRect().height}px`;
			svg.style.cssText = `display:block;width:${width}px;height:${height}px;max-width:none;`;
			canvas.style.width = `${width}px`;
			canvas.style.height = `${height}px`;
			canvas.append(svg);
			originalOverflow = document.documentElement.style.overflow;
			document.documentElement.style.overflow = 'hidden';
			dialog.showModal();
			actual();
		};
		dialog.addEventListener('close', () => {
			if (source && svg) {
				if (originalStyle === null) svg.removeAttribute('style');
				else svg.setAttribute('style', originalStyle);
				source.append(svg);
				source.style.removeProperty('min-height');
				source.scrollLeft = originalScroll.x;
				source.scrollTop = originalScroll.y;
			}
			document.documentElement.style.overflow = originalOverflow;
			pointers.clear();
			startGesture();
			opener?.focus({ preventScroll: true });
			source = undefined;
			svg = undefined;
		});
		this.querySelector('[data-action="close"]')!.addEventListener('click', () => dialog.close());
		zoomIn.addEventListener('click', () => zoom(view.scale * 1.25));
		zoomOut.addEventListener('click', () => zoom(view.scale / 1.25));
		this.querySelector('[data-action="fit"]')!.addEventListener('click', fit);
		this.querySelector('[data-action="actual"]')!.addEventListener('click', actual);
		// Keep Tab in the viewer even in browsers that otherwise visit their
		// chrome after the native dialog's last focus stop.
		dialog.addEventListener('keydown', event => {
			if (event.key !== 'Tab' || event.ctrlKey || event.metaKey || event.altKey) return;
			const stops = [...dialog.querySelectorAll<HTMLElement>('button:not(:disabled), [tabindex="0"], a[href]')];
			const target = event.shiftKey ? stops.at(-1)! : stops[0];
			if (document.activeElement === (event.shiftKey ? stops[0] : stops.at(-1))) {
				event.preventDefault();
				target.focus();
			}
		});
		// A backdrop click dismisses; dragging or interacting inside never does.
		let backdropDown = false;
		dialog.addEventListener('pointerdown', event => { backdropDown = event.target === dialog; });
		dialog.addEventListener('click', event => {
			if (backdropDown && event.target === dialog) dialog.close();
			backdropDown = false;
		});
		stage.addEventListener('pointerdown', event => {
			if (event.button !== 0) return;
			stage.focus({ preventScroll: true });
			pointers.set(event.pointerId, point(event));
			stage.setPointerCapture(event.pointerId);
			startGesture();
		});
		stage.addEventListener('pointermove', event => {
			if (!pointers.has(event.pointerId) || !gesture) return;
			pointers.set(event.pointerId, point(event));
			const { midpoint, distance } = geometry();
			if (gesture.distance > 0 && pointers.size > 1) {
				zoom(gesture.view.scale * distance / gesture.distance, gesture.midpoint, gesture.view, { x: midpoint.x - gesture.midpoint.x, y: midpoint.y - gesture.midpoint.y });
			} else {
				paint({ ...gesture.view, x: gesture.view.x + midpoint.x - gesture.midpoint.x, y: gesture.view.y + midpoint.y - gesture.midpoint.y });
			}
		});
		const endPointer = (event: PointerEvent) => {
			pointers.delete(event.pointerId);
			startGesture();
		};
		stage.addEventListener('pointerup', endPointer);
		stage.addEventListener('pointercancel', endPointer);
		stage.addEventListener('lostpointercapture', endPointer);
		stage.addEventListener('wheel', event => {
			event.preventDefault();
			const delta = event.deltaY * (event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? stage.clientHeight : 1);
			zoom(view.scale * Math.exp(-delta * 0.002), point(event));
		}, { passive: false });
		stage.addEventListener('keydown', event => {
			if (event.ctrlKey || event.metaKey || event.altKey) return;
			if (event.key === '+' || event.key === '=') zoom(view.scale * 1.25);
			else if (event.key === '-') zoom(view.scale / 1.25);
			else if (event.key === '0' || event.key === 'Home') fit();
			else if (event.key.startsWith('Arrow')) {
				paint({ ...view, x: view.x + (event.key === 'ArrowLeft' ? 48 : event.key === 'ArrowRight' ? -48 : 0), y: view.y + (event.key === 'ArrowUp' ? 48 : event.key === 'ArrowDown' ? -48 : 0) });
			} else return;
			event.preventDefault();
		});
		new ResizeObserver(() => {
			if (dialog.open) paint({ ...view, scale: Math.max(fitScale(), view.scale) });
		}).observe(stage);

		document.querySelectorAll<HTMLPreElement>('.sl-markdown-content pre.mermaid').forEach(pre => {
			const frame = document.createElement('div');
			frame.className = 'diagram-frame';
			pre.before(frame);
			frame.append(pre);
			const enhance = () => {
				const diagram = pre.querySelector<SVGSVGElement>('svg');
				if (!diagram) return false;
				const toolbar = template.content.cloneNode(true) as DocumentFragment;
				const button = toolbar.querySelector<HTMLButtonElement>('button')!;
				const caption = diagram.querySelector('title')?.textContent || title.textContent!;
				toolbar.querySelector('.diagram-caption')!.textContent = caption;
				button.dataset.title = caption;
				button.setAttribute('aria-label', `${button.textContent!.trim()}: ${caption}`);
				button.addEventListener('click', () => open(pre, button));
				pre.before(toolbar);
				pre.tabIndex = 0;
				pre.setAttribute('role', 'region');
				pre.setAttribute('aria-label', caption);
				// Preserve a natural 16px label size; only wide diagrams scroll.
				diagram.style.width = `${diagram.viewBox.baseVal.width}px`;
				diagram.style.maxWidth = 'none';
				return true;
			};
			if (!enhance()) {
				const observer = new MutationObserver(() => { if (enhance()) observer.disconnect(); });
				observer.observe(pre, { childList: true });
			}
		});
	}
}

if (!customElements.get('tspm-diagram-viewer')) customElements.define('tspm-diagram-viewer', DiagramViewer);
