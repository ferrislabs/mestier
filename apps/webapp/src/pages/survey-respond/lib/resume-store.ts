export function resumeKey(slug: string): string {
	return `mestier.survey.resume.${slug}`
}

export function readResumeToken(slug: string): string | null {
	try {
		return window.localStorage.getItem(resumeKey(slug))
	} catch {
		return null
	}
}

export function saveResumeToken(slug: string, token: string): void {
	try {
		window.localStorage.setItem(resumeKey(slug), token)
	} catch {
		return
	}
}

export function forgetResumeToken(slug: string): void {
	try {
		window.localStorage.removeItem(resumeKey(slug))
	} catch {
		return
	}
}
