import { Plus, Trash2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import {
	CADENCE_KINDS,
	type CadenceDraft,
	type CadenceKind,
	MAX_WINDOWS,
} from '#/pages/maintenance/lib/cadence-form'

export interface CadenceFieldProps {
	/** Makes the labels unique when several lines are on screen. */
	idPrefix: string
	value: CadenceDraft
	onChange: (value: CadenceDraft) => void
}

/**
 * The "périodicité" picker: every N days, windows of weeks, or on demand.
 */
export function CadenceField({ idPrefix, value, onChange }: CadenceFieldProps) {
	function setWindow(
		index: number,
		patch: Partial<CadenceDraft['windows'][0]>,
	) {
		onChange({
			...value,
			windows: value.windows.map((window, at) =>
				at === index ? { ...window, ...patch } : window,
			),
		})
	}

	return (
		<div className="space-y-2">
			<div className="flex flex-wrap items-center gap-2">
				<select
					id={`${idPrefix}-cadence`}
					aria-label="Périodicité"
					className="h-9 rounded-md border border-input bg-transparent px-3 text-sm"
					value={value.kind}
					onChange={(event) => {
						const kind = event.target.value as CadenceKind
						onChange({
							...value,
							kind,
							// A first, empty window to type into.
							windows:
								kind === 'WINDOWS' && value.windows.length === 0
									? [{ from: '', to: '' }]
									: value.windows,
						})
					}}
				>
					{CADENCE_KINDS.map((entry) => (
						<option key={entry.kind} value={entry.kind}>
							{entry.label}
						</option>
					))}
				</select>

				{value.kind === 'ROLLING' ? (
					<div className="flex items-center gap-2 text-sm">
						<span>Tous les</span>
						<Input
							aria-label="Nombre de jours"
							inputMode="numeric"
							className="w-20"
							value={value.everyDays}
							onChange={(event) =>
								onChange({ ...value, everyDays: event.target.value })
							}
						/>
						<span>jours, depuis la dernière intervention validée</span>
					</div>
				) : null}

				{value.kind === 'ON_DEMAND' ? (
					<span className="text-sm text-muted-foreground">
						Jamais due. Les interventions sont enregistrées quand même.
					</span>
				) : null}
			</div>

			{value.kind === 'WINDOWS' ? (
				<div className="space-y-2">
					{value.windows.map((window, index) => (
						// biome-ignore lint/suspicious/noArrayIndexKey: windows have no id, they are edited in place
						<div key={index} className="flex items-center gap-2 text-sm">
							<span>De la semaine</span>
							<Input
								aria-label={`Fenêtre ${index + 1}, première semaine`}
								inputMode="numeric"
								className="w-16"
								value={window.from}
								onChange={(event) =>
									setWindow(index, { from: event.target.value })
								}
							/>
							<span>à la semaine</span>
							<Input
								aria-label={`Fenêtre ${index + 1}, dernière semaine`}
								inputMode="numeric"
								className="w-16"
								value={window.to}
								onChange={(event) =>
									setWindow(index, { to: event.target.value })
								}
							/>
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								aria-label={`Retirer la fenêtre ${index + 1}`}
								onClick={() =>
									onChange({
										...value,
										windows: value.windows.filter((_, at) => at !== index),
									})
								}
							>
								<Trash2 />
							</Button>
						</div>
					))}
					<Button
						type="button"
						variant="outline"
						size="sm"
						disabled={value.windows.length >= MAX_WINDOWS}
						onClick={() =>
							onChange({
								...value,
								windows: [...value.windows, { from: '', to: '' }],
							})
						}
					>
						<Plus />
						Ajouter une fenêtre
					</Button>
				</div>
			) : null}
		</div>
	)
}
