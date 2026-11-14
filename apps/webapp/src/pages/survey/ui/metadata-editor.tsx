import { Plus, X } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import type { MetadataEntry } from '#/pages/survey/lib/metadata'

export function MetadataEditor({
	entries,
	errors,
	readOnly,
	onChange,
	onAdd,
}: {
	entries: readonly MetadataEntry[]
	errors: readonly string[]
	readOnly: boolean
	onChange: (entries: MetadataEntry[]) => void
	onAdd: () => void
}) {
	const update = (id: string, patch: Partial<MetadataEntry>) =>
		onChange(
			entries.map((entry) =>
				entry.id === id ? { ...entry, ...patch } : entry,
			),
		)
	return (
		<div className="grid gap-2">
			{entries.length === 0 ? (
				<p className="text-muted-foreground text-xs">Aucune métadonnée.</p>
			) : (
				<ul className="grid gap-2">
					{entries.map((entry, position) => (
						<li key={entry.id} className="flex gap-2">
							<Input
								aria-label={`Clé ${position + 1}`}
								placeholder="Clé"
								value={entry.key}
								disabled={readOnly}
								onChange={(event) =>
									update(entry.id, { key: event.target.value })
								}
							/>
							<Input
								aria-label={`Valeur ${position + 1}`}
								placeholder="Valeur"
								value={entry.value}
								disabled={readOnly}
								onChange={(event) =>
									update(entry.id, { value: event.target.value })
								}
							/>
							{readOnly ? null : (
								<Button
									variant="ghost"
									size="icon"
									aria-label={`Retirer la métadonnée ${position + 1}`}
									onClick={() =>
										onChange(entries.filter((item) => item.id !== entry.id))
									}
								>
									<X aria-hidden />
								</Button>
							)}
						</li>
					))}
				</ul>
			)}
			{errors.map((message) => (
				<p key={message} className="text-destructive text-xs">
					{message}
				</p>
			))}
			{readOnly ? null : (
				<Button variant="outline" size="sm" className="w-fit" onClick={onAdd}>
					<Plus aria-hidden />
					Ajouter une métadonnée
				</Button>
			)}
		</div>
	)
}
