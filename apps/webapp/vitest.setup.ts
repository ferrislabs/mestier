import { cleanup } from '@testing-library/react'
import { afterEach } from 'vitest'

// jsdom does not implement scrollTo; Radix Popper positioning (used by popovers,
// dropdowns, selects, etc.) calls it when computing layout on open.
window.scrollTo = () => {}

afterEach(() => {
	cleanup()
})
