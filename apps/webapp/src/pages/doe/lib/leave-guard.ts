type Place = { pathname: string }

function normalized(pathname: string): string {
	return pathname.length > 1 ? pathname.replace(/\/+$/, '') : pathname
}

export function leavesEditor({
	current,
	next,
}: {
	current: Place
	next: Place
}): boolean {
	return normalized(current.pathname) !== normalized(next.pathname)
}
