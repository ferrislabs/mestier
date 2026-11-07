import { type FormEvent, useId, useState } from 'react'
import { Input } from '#/components/ui/input'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import {
	evaluateResume,
	type SequenceRow,
} from '#/pages/crm-settings/lib/settings-model'
import type { SequenceKind } from '#/pages/crm-settings/types'
import {
	SettingsCard,
	SettingsForm,
} from '#/pages/crm-settings/ui/settings-card'

const RULES: readonly string[] = [
	'Les numéros sont attribués à l’émission, dans l’ordre chronologique, sans trou.',
	'Un brouillon n’a pas de numéro et peut être supprimé.',
	'Une facture émise ne peut plus être modifiée ni supprimée : on la corrige ou on l’annule par un avoir.',
	'Les avoirs ont leur propre série.',
]

type Inputs = Record<SequenceKind, string>

const NO_INPUTS: Inputs = { quote: '', invoice: '', creditNote: '' }

export function NumberingSection({
	rows,
	padding,
	error,
	onSave,
}: {
	rows: readonly SequenceRow[]
	padding: number
	error: string | null
	onSave: (
		resumes: readonly { kind: SequenceKind; nextNumber: number }[],
	) => Promise<void>
}) {
	const baseId = useId()
	const [inputs, setInputs] = useState<Inputs>(NO_INPUTS)
	const [saved, setSaved] = useState(false)
	const results = rows.map((row) => ({
		row,
		result: evaluateResume(row, inputs[row.kind], padding),
	}))
	const dirty = results.some(({ row }) => inputs[row.kind].trim() !== '')
	const blocked = results.some(({ result }) => result.error !== null)

	function change(kind: SequenceKind, value: string) {
		setInputs((current) => ({ ...current, [kind]: value }))
		setSaved(false)
	}

	function submit(event: FormEvent) {
		event.preventDefault()
		if (blocked) {
			const first = results.find(({ result }) => result.error !== null)
			if (first) document.getElementById(`${baseId}-${first.row.kind}`)?.focus()
			return
		}
		const resumes = results.flatMap(({ row, result }) =>
			result.nextNumber === null
				? []
				: [{ kind: row.kind, nextNumber: result.nextNumber }],
		)
		void onSave(resumes).then(
			() => {
				setInputs(NO_INPUTS)
				setSaved(true)
			},
			() => setSaved(false),
		)
	}

	return (
		<SettingsCard
			id="numerotation"
			title="Numérotation"
			description="Chaque série a son compteur, qui repart de 1 chaque année."
		>
			<SettingsForm
				onSubmit={submit}
				dirty={dirty}
				saved={saved}
				savedLabel="Compteurs mis à jour."
				error={error}
			>
				<div className="overflow-hidden rounded-md border">
					<Table>
						<TableHeader>
							<TableRow className="hover:bg-transparent">
								<TableHead className="pl-3">Série</TableHead>
								<TableHead>Préfixe</TableHead>
								<TableHead>Année</TableHead>
								<TableHead>Dernier numéro attribué</TableHead>
								<TableHead>Prochain numéro</TableHead>
								<TableHead className="pr-3">Reprendre à partir de…</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{results.map(({ row, result }) => {
								const inputId = `${baseId}-${row.kind}`
								const hintId = `${inputId}-hint`
								return (
									<TableRow key={row.kind}>
										<TableCell className="py-3 pl-3 font-medium">
											{row.label}
										</TableCell>
										<TableCell className="font-mono text-[13px]">
											{row.prefix}
										</TableCell>
										<TableCell className="tabular">{row.year}</TableCell>
										<TableCell className="font-mono text-[13px]">
											{row.lastNumber ?? (
												<span className="font-sans text-muted-foreground">
													Aucun
												</span>
											)}
										</TableCell>
										<TableCell className="font-mono text-[13px]">
											{row.next}
										</TableCell>
										<TableCell className="min-w-56 py-2 pr-3">
											<label htmlFor={inputId} className="sr-only">
												Reprendre la série {row.label} à partir du numéro
											</label>
											<Input
												id={inputId}
												inputMode="numeric"
												autoComplete="off"
												placeholder="Numéro suivant"
												value={inputs[row.kind]}
												onChange={(event) =>
													change(row.kind, event.target.value)
												}
												aria-invalid={result.error ? true : undefined}
												aria-describedby={hintId}
												className="h-8 w-full tabular"
											/>
											<p
												id={hintId}
												className={
													result.error
														? 'mt-1 text-[12px] text-state-alert'
														: 'mt-1 text-[12px] text-muted-foreground'
												}
											>
												{result.error ??
													(result.preview
														? `Le prochain numéro sera ${result.preview}.`
														: '')}
											</p>
										</TableCell>
									</TableRow>
								)
							})}
						</TableBody>
					</Table>
				</div>
				<p className="text-[13px] text-muted-foreground">
					Saisis le numéro que doit porter le prochain document, par exemple
					pour reprendre une numérotation commencée dans un autre outil. Un
					compteur ne peut qu’avancer, jamais reculer : un numéro inférieur ou
					égal au prochain numéro est refusé, car il créerait des doublons.
				</p>
				<div>
					<h3 className="mb-2 font-medium text-sm">Règles de numérotation</h3>
					<ul className="grid list-disc gap-1 pl-5 text-[13px] text-muted-foreground">
						{RULES.map((rule) => (
							<li key={rule}>{rule}</li>
						))}
					</ul>
				</div>
			</SettingsForm>
		</SettingsCard>
	)
}
