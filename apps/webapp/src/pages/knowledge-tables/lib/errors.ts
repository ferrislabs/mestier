import { isForbiddenError, isNotFoundError } from '#/lib/api-error'

const KNOWN_REFUSALS: readonly [string, string][] = [
	[
		'a CSV file holds at most 5 MB',
		'Le fichier dépasse 5 Mo : rien n’a été importé.',
	],
	[
		'a CSV file holds at most 10000 rows',
		'Le fichier compte plus de 10 000 lignes : rien n’a été importé.',
	],
	[
		'becomes a table of at most 50 columns',
		'Le fichier compte plus de 50 colonnes : rien n’a été importé.',
	],
	[
		'the CSV file has no header line',
		'Le fichier est vide ou n’a pas de ligne d’en-tête.',
	],
	['cannot be read as CSV', 'Ce fichier ne se lit pas comme un CSV.'],
	[
		'no column of the file matches',
		'Aucun en-tête du fichier ne porte le nom d’une colonne de la table.',
	],
	['a table with this name already exists', 'Une table porte déjà ce nom.'],
	[
		'a column with this name already exists',
		'Une colonne porte déjà ce nom dans la table.',
	],
	[
		'a view with this name already exists',
		'Une vue porte déjà ce nom dans la table.',
	],
	['a name cannot be empty', 'Le nom ne peut pas être vide.'],
	['a name holds at most 100', 'Un nom compte 100 caractères au plus.'],
	[
		'a description holds at most',
		'Une description compte 2 000 caractères au plus.',
	],
	['at most 200 tables', 'Une organisation compte 200 tables au plus.'],
	['at most 50 columns', 'Une table compte 50 colonnes au plus.'],
	['at most 10000 rows', 'Une table compte 10 000 lignes au plus.'],
	['at most 20 views', 'Une table compte 20 vues au plus.'],
	['at most 100 options', 'Une colonne de choix compte 100 options au plus.'],
	['an option label cannot be empty', 'Une option doit avoir un libellé.'],
	['two options share a label', 'Deux options portent le même libellé.'],
	['0 to 4 decimals', 'Un nombre affiche de 0 à 4 décimales.'],
	[
		'the primary column is never deleted',
		'La colonne principale ne se supprime pas et reste un texte.',
	],
	[
		'this change of type would lose values',
		'Ce changement de type perdrait des valeurs : crée plutôt une nouvelle colonne.',
	],
	[
		'a stored value does not fit the new type',
		'Une valeur ne tient pas dans le nouveau type : rien n’a changé.',
	],
	['a table keeps at least one view', 'Une table garde au moins une vue.'],
	[
		'a kanban groups by a single choice column',
		'Un Kanban se groupe par une colonne de choix unique.',
	],
	[
		'link, file and member columns cannot be sorted',
		'Un lien, un fichier ou un membre ne se trie pas.',
	],
	['sorts on at most 3 columns', 'Une vue trie sur 3 colonnes au plus.'],
	['at most 20 filters', 'Une vue compte 20 filtres au plus.'],
	['the value does not fit the column', 'La valeur du filtre ne convient pas.'],
	[
		'this operator does not apply',
		'Ce filtre ne s’applique pas à cette colonne.',
	],
	['the text is too long', 'Le texte est trop long pour la colonne.'],
	['expected a number', 'Saisis un nombre.'],
	['expected a whole number of cents', 'Saisis un montant en euros.'],
	['expected a date', 'Saisis une date valide.'],
	['no such option', 'Cette option n’existe plus.'],
	['an option is listed twice', 'Une option est choisie deux fois.'],
	[
		'another kind of record',
		'Ce lien vise un autre genre d’élément que la colonne.',
	],
	['not found in the organization', 'Cet élément est introuvable.'],
	['no such column', 'Cette colonne n’existe plus.'],
	[
		'at most 1000 rows are deleted',
		'On supprime 1 000 lignes au plus à la fois.',
	],
]

export function tableErrorMessage(error: unknown): string | null {
	if (error == null) return null
	if (isForbiddenError(error)) {
		return "Vous n'avez plus la permission nécessaire pour cette action."
	}
	if (isNotFoundError(error)) return 'Cet élément n’existe plus.'
	const message = error instanceof Error ? error.message : ''
	const known = KNOWN_REFUSALS.find(([fragment]) => message.includes(fragment))
	return known?.[1] ?? (message || 'Une erreur est survenue.')
}
