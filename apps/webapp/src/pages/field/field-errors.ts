/**
 * Turns a backend conflict message into something a worker on a phone can
 * read.
 *
 * The backend keeps its messages in English on purpose, for logs, and the
 * product decision was to map them here rather than add an error-code field
 * to the shared `ApiError` contract every other module also uses (see
 * `docs/projets/ux-simplification.md`). `ApiError`'s wire format wraps a
 * `CoreError::Conflict` in a `"Conflict: {0}"` display, so matching is done
 * with `includes` rather than an exact comparison — a wording change on the
 * backend still fails loudly here (it stops matching) rather than silently,
 * it just does so without being coupled to that prefix.
 */
const KNOWN_MESSAGES: { needle: string; translation: string }[] = [
	{
		needle: 'a time entry cannot end before it started',
		translation: "L'heure de fin doit être après l'heure de début.",
	},
	{
		needle:
			'this stretch began on an earlier day and needs its end time declared',
		translation:
			"Ce pointage a commencé un autre jour : indiquez l'heure de fin réelle.",
	},
	{
		needle:
			'this stretch is from today, so it is stopped rather than recovered',
		translation:
			"Ce pointage date d'aujourd'hui : utilisez plutôt le bouton d'arrêt normal.",
	},
	{
		needle: 'time entry is already closed',
		translation: 'Ce pointage est déjà clôturé.',
	},
	{
		needle: 'employee is already clocked on to task ',
		translation: 'Vous êtes déjà pointé sur un autre projet.',
	},
	{
		needle: 'a time entry cannot end in the future',
		translation: "L'heure de fin ne peut pas être dans le futur.",
	},
	{
		needle:
			'declared stretch overlaps the job the employee is still clocked on to',
		translation:
			'Ce temps chevauche un chantier sur lequel vous êtes encore pointé. Clôturez-le avant de déclarer ce temps.',
	},
	{
		needle: 'a resolved report can no longer be amended',
		translation:
			'Ce signalement a déjà été traité par un responsable, vous ne pouvez plus le modifier.',
	},
	{
		needle: 'a resolved report can no longer be withdrawn',
		translation:
			'Ce signalement a déjà été traité par un responsable, vous ne pouvez plus le retirer.',
	},
	{
		needle: 'uq_assignment_reports_pending_per_assignment',
		translation:
			'Vous avez déjà un signalement en attente sur ce projet — modifiez-le plutôt que d’en créer un second.',
	},
]

const GENERIC_FALLBACK = 'Une erreur est survenue, réessayez.'

/** French text for a raw backend error message, never the English original. */
export function fieldErrorMessage(message: string): string {
	const known = KNOWN_MESSAGES.find(({ needle }) => message.includes(needle))
	return known?.translation ?? GENERIC_FALLBACK
}
