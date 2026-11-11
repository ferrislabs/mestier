import { Plus, X } from 'lucide-react'
import { useId } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { cn } from '#/lib/utils'
import type {
	ColumnDraft,
	OptionDraft,
} from '#/pages/knowledge-tables/lib/column-draft'
import {
	DECIMAL_CHOICES,
	kindLabel,
	LINK_KINDS,
	linkKindLabel,
	SELECT_COLOR_CLASSES,
	SELECT_COLOR_LABELS,
	SELECT_COLORS,
} from '#/pages/knowledge-tables/lib/columns'
import type {
	ColumnKind,
	LinkKind,
	SelectColor,
} from '#/pages/knowledge-tables/lib/types'
import { NativeSelect } from '#/pages/knowledge-tables/ui/native-select'

interface ColumnDialogProps {
	open: boolean
	mode: 'create' | 'edit'
	draft: ColumnDraft
	kinds: readonly ColumnKind[]
	isPrimary: boolean
	error: string | null
	isSaving: boolean
	onDraftChange: (change: Partial<ColumnDraft>) => void
	onAddOption: () => void
	onSubmit: () => void
	onOpenChange: (open: boolean) => void
}

function OptionsEditor({
	options,
	onChange,
	onAdd,
}: {
	options: readonly OptionDraft[]
	onChange: (options: OptionDraft[]) => void
	onAdd: () => void
}) {
	const replace = (key: string, change: Partial<OptionDraft>) =>
		onChange(
			options.map((option) =>
				option.key === key ? { ...option, ...change } : option,
			),
		)
	return (
		<fieldset className="grid gap-2">
			<legend className="mb-1.5 font-medium text-sm">Options</legend>
			{options.length === 0 ? (
				<p className="text-muted-foreground text-sm">
					Aucune option pour l’instant.
				</p>
			) : (
				<ul className="grid max-h-64 gap-1.5 overflow-y-auto pr-1">
					{options.map((option, index) => (
						<li
							key={option.key}
							className="grid grid-cols-[7rem_minmax(0,1fr)_auto] items-center gap-2"
						>
							<NativeSelect
								aria-label={`Couleur de l’option ${index + 1}`}
								value={option.color}
								className={cn(
									'font-medium',
									SELECT_COLOR_CLASSES[option.color],
								)}
								onChange={(event) =>
									replace(option.key, {
										color: event.target.value as SelectColor,
									})
								}
							>
								{SELECT_COLORS.map((color) => (
									<option key={color} value={color}>
										{SELECT_COLOR_LABELS[color]}
									</option>
								))}
							</NativeSelect>
							<Input
								aria-label={`Libellé de l’option ${index + 1}`}
								value={option.label}
								maxLength={100}
								className="h-8 text-[13px]"
								onChange={(event) =>
									replace(option.key, { label: event.target.value })
								}
							/>
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								aria-label={`Retirer l’option ${option.label || index + 1}`}
								onClick={() =>
									onChange(options.filter((entry) => entry.key !== option.key))
								}
							>
								<X aria-hidden />
							</Button>
						</li>
					))}
				</ul>
			)}
			<div>
				<Button type="button" variant="outline" size="sm" onClick={onAdd}>
					<Plus aria-hidden />
					Ajouter une option
				</Button>
			</div>
			<p className="text-muted-foreground text-xs">
				Retirer une option la retire aussi des lignes qui l’ont.
			</p>
		</fieldset>
	)
}

export function ColumnDialog({
	open,
	mode,
	draft,
	kinds,
	isPrimary,
	error,
	isSaving,
	onDraftChange,
	onAddOption,
	onSubmit,
	onOpenChange,
}: ColumnDialogProps) {
	const baseId = useId()
	const isChoice =
		draft.kind === 'single_select' || draft.kind === 'multi_select'

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="sm:max-w-lg">
				<form
					className="grid gap-4"
					onSubmit={(event) => {
						event.preventDefault()
						onSubmit()
					}}
				>
					<DialogHeader>
						<DialogTitle>
							{mode === 'create' ? 'Nouvelle colonne' : 'Modifier la colonne'}
						</DialogTitle>
						<DialogDescription>
							{isPrimary
								? 'La colonne principale donne son titre à chaque ligne : elle reste un texte.'
								: mode === 'edit'
									? 'Seuls les changements de type sans perte sont proposés.'
									: 'Choisis le type des valeurs de la colonne.'}
						</DialogDescription>
					</DialogHeader>
					<div className="grid gap-1.5">
						<Label htmlFor={`${baseId}-name`}>Nom</Label>
						<Input
							id={`${baseId}-name`}
							value={draft.name}
							maxLength={100}
							autoComplete="off"
							autoFocus
							onChange={(event) => onDraftChange({ name: event.target.value })}
						/>
					</div>
					<div className="grid gap-1.5">
						<Label htmlFor={`${baseId}-kind`}>Type</Label>
						<NativeSelect
							id={`${baseId}-kind`}
							value={draft.kind}
							disabled={kinds.length <= 1}
							onChange={(event) =>
								onDraftChange({ kind: event.target.value as ColumnKind })
							}
						>
							{kinds.map((kind) => (
								<option key={kind} value={kind}>
									{kindLabel(kind)}
								</option>
							))}
						</NativeSelect>
					</div>
					{draft.kind === 'number' ? (
						<div className="grid gap-1.5">
							<Label htmlFor={`${baseId}-decimals`}>Décimales affichées</Label>
							<NativeSelect
								id={`${baseId}-decimals`}
								value={draft.decimals}
								onChange={(event) =>
									onDraftChange({ decimals: Number(event.target.value) })
								}
							>
								{DECIMAL_CHOICES.map((count) => (
									<option key={count} value={count}>
										{count === 0 ? 'Aucune' : count}
									</option>
								))}
							</NativeSelect>
						</div>
					) : null}
					{draft.kind === 'link' ? (
						<div className="grid gap-1.5">
							<Label htmlFor={`${baseId}-link`}>Lien vers</Label>
							{mode === 'create' ? (
								<NativeSelect
									id={`${baseId}-link`}
									value={draft.linkKind}
									onChange={(event) =>
										onDraftChange({ linkKind: event.target.value as LinkKind })
									}
								>
									{LINK_KINDS.map((entry) => (
										<option key={entry.kind} value={entry.kind}>
											{entry.label}
										</option>
									))}
								</NativeSelect>
							) : (
								<p id={`${baseId}-link`} className="text-sm">
									{linkKindLabel(draft.linkKind)}, fixé à la création.
								</p>
							)}
						</div>
					) : null}
					{isChoice ? (
						<OptionsEditor
							options={draft.options}
							onChange={(options) => onDraftChange({ options })}
							onAdd={onAddOption}
						/>
					) : null}
					{error ? (
						<p role="alert" className="text-[13px] text-state-alert">
							{error}
						</p>
					) : null}
					<DialogFooter>
						<Button
							type="button"
							variant="outline"
							onClick={() => onOpenChange(false)}
						>
							Annuler
						</Button>
						<Button
							type="submit"
							disabled={isSaving || error !== null || draft.name.trim() === ''}
						>
							{mode === 'create' ? 'Ajouter' : 'Enregistrer'}
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}
