export function installFlowTestEnvironment() {
	if (typeof document.elementFromPoint !== 'function') {
		document.elementFromPoint = () => null
	}

	if (
		typeof (window as unknown as { DOMMatrixReadOnly?: unknown })
			.DOMMatrixReadOnly !== 'function'
	) {
		class FallbackDOMMatrixReadOnly {
			m22 = 1
			constructor(transform?: string) {
				if (!transform || transform === 'none') return
				const match = /matrix\(([^)]+)\)/.exec(transform)
				if (!match) return
				const parts = match[1]
					.split(',')
					.map((part) => Number.parseFloat(part.trim()))
				if (parts.length >= 4 && Number.isFinite(parts[3])) this.m22 = parts[3]
			}
		}
		;(window as unknown as { DOMMatrixReadOnly: unknown }).DOMMatrixReadOnly =
			FallbackDOMMatrixReadOnly
	}

	window.ResizeObserver = class {
		callback: ResizeObserverCallback
		constructor(callback: ResizeObserverCallback) {
			this.callback = callback
		}
		observe(target: Element) {
			const size = { inlineSize: 150, blockSize: 40 }
			this.callback(
				[
					{
						target,
						contentRect: target.getBoundingClientRect(),
						borderBoxSize: [size],
						contentBoxSize: [size],
						devicePixelContentBoxSize: [size],
					} as unknown as ResizeObserverEntry,
				],
				this as unknown as ResizeObserver,
			)
		}
		unobserve() {}
		disconnect() {}
	}

	Object.defineProperty(HTMLElement.prototype, 'offsetWidth', {
		configurable: true,
		get: () => 150,
	})
	Object.defineProperty(HTMLElement.prototype, 'offsetHeight', {
		configurable: true,
		get: () => 40,
	})

	Element.prototype.getBoundingClientRect = () =>
		({
			width: 2000,
			height: 1200,
			top: 0,
			left: 0,
			right: 2000,
			bottom: 1200,
			x: 0,
			y: 0,
			toJSON() {},
		}) as DOMRect
}

function fireMouseEvent(
	target: EventTarget,
	type: string,
	clientX: number,
	clientY: number,
) {
	const event = new MouseEvent(type, {
		clientX,
		clientY,
		bubbles: true,
		cancelable: true,
		button: 0,
	})
	Object.defineProperty(event, 'view', { value: window, configurable: true })
	target.dispatchEvent(event)
}

export function dragNodeBy(node: HTMLElement, dx: number, dy: number) {
	const startX = 100
	const startY = 100
	const armX = startX + 5
	const armY = startY + 5
	fireMouseEvent(node, 'mousedown', startX, startY)
	fireMouseEvent(window, 'mousemove', armX, armY)
	fireMouseEvent(window, 'mousemove', armX + dx, armY + dy)
	fireMouseEvent(window, 'mouseup', armX + dx, armY + dy)
}
