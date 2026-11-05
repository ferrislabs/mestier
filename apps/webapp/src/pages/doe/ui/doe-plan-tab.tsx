import { ArrowDown, ArrowUp, Plus, Trash2 } from 'lucide-react'
import { useId, useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { cn } from '#/lib/utils'
import { numberSections } from '#/pages/doe/lib/sections'
import {
	type DoeSection,
	KIND_LABELS,
	SECTION_KINDS,
	type SectionKind,
} from '#/pages/doe/types'
import { EditorCard } from '#/pages/doe/ui/editor-card'

export const REMOVE_BLOCKED_NOTE =
	'Ce thème contient des éléments. Retire-les avant de le supprimer, ou décoche le thème pour l’exclure du DOE.'

interface DoePlanTabProps {
	sections: DoeSection[]
	removableSectionIds: ReadonlySet<string>
	onToggle: (id: string, enabled: boolean) => void
	onRename: (id: string, title: string) => void
	onMove: (index: number, delta: -1 | 1) => void
	onRemove: (id: string) => void
	onAdd: (title: string, kind: SectionKind) => void
}

export function DoePlanTab({
	sections,
	removableSectionIds,
	onToggle,
	onRename,
	onMove,
	onRemove,
	onAdd,
}: DoePlanTabProps) {
	const base = useId()
	const [newTitle, setNewTitle] = useState('')
	const [newKind, setNewKind] = useState<SectionKind>('supplies')
	const numbered = numberSections(sections)

	return (
		<EditorCard
			title="Thèmes"
			description="Coche les thèmes à inclure. La numérotation ne compte que les thèmes cochés."
		>
			<ul className="divide-y">
				{numbered.map(({ section, number }, index) => {
					const removable = removableSectionIds.has(section.id)
					return (
						<li
							key={section.id}
							className="flex flex-wrap items-center gap-3 px-5 py-3"
						>
							<input
								type="checkbox"
								checked={section.enabled}
								aria-label={`Inclure ${section.title}`}
								onChange={(event) => onToggle(section.id, event.target.checked)}
								className="size-4 rounded-sm border-input accent-primary outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
							/>
							<span className="w-6 text-center text-muted-foreground text-sm tabular-nums">
								{number === null ? (
									<>
										<span aria-hidden>—</span>
										<span className="sr-only">Thème non numéroté</span>
									</>
								) : (
									<>
										<span className="sr-only">Thème n° </span>
										{number}
									</>
								)}
							</span>
							<Input
								value={section.title}
								aria-label={`Titre du thème ${index + 1}`}
								aria-invalid={section.title.trim() === ''}
								onChange={(event) => onRename(section.id, event.target.value)}
								className="min-w-48 max-w-sm flex-1"
							/>
							<Badge variant="secondary">
								{KIND_LABELS[section.content.kind]}
							</Badge>
							<div className="ml-auto flex items-center gap-1">
								<Button
									type="button"
									variant="ghost"
									size="icon-sm"
									disabled={index === 0}
									onClick={() => onMove(index, -1)}
								>
									<ArrowUp aria-hidden />
									<span className="sr-only">Monter {section.title}</span>
								</Button>
								<Button
									type="button"
									variant="ghost"
									size="icon-sm"
									disabled={index === sections.length - 1}
									onClick={() => onMove(index, 1)}
								>
									<ArrowDown aria-hidden />
									<span className="sr-only">Descendre {section.title}</span>
								</Button>
								<Button
									type="button"
									variant="ghost"
									size="icon-sm"
									aria-disabled={!removable}
									title={removable ? undefined : REMOVE_BLOCKED_NOTE}
									className={cn(
										!removable &&
											'cursor-not-allowed opacity-50 hover:bg-transparent hover:text-current',
									)}
									onClick={() => {
										if (removable) onRemove(section.id)
									}}
								>
									<Trash2 aria-hidden />
									<span className="sr-only">Supprimer {section.title}</span>
								</Button>
							</div>
						</li>
					)
				})}
			</ul>
			<RequirePermission permission="MANAGE_PLANNING">
				<form
					className="flex flex-wrap items-end gap-3 border-t p-5"
					onSubmit={(event) => {
						event.preventDefault()
						if (newTitle.trim() === '') return
						onAdd(newTitle, newKind)
						setNewTitle('')
					}}
				>
					<div className="grid gap-1.5">
						<label htmlFor={`${base}-title`} className="font-medium text-sm">
							Titre du nouveau thème
						</label>
						<Input
							id={`${base}-title`}
							value={newTitle}
							className="w-64"
							onChange={(event) => setNewTitle(event.target.value)}
						/>
					</div>
					<div className="grid gap-1.5">
						<span className="font-medium text-sm">Type</span>
						<Select
							value={newKind}
							onValueChange={(value) => {
								const found = SECTION_KINDS.find((item) => item === value)
								if (found) setNewKind(found)
							}}
						>
							<SelectTrigger
								aria-label="Type du nouveau thème"
								className="w-44"
							>
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								{SECTION_KINDS.map((kind) => (
									<SelectItem key={kind} value={kind}>
										{KIND_LABELS[kind]}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</div>
					<Button type="submit" disabled={newTitle.trim() === ''}>
						<Plus aria-hidden />
						Ajouter un thème
					</Button>
				</form>
			</RequirePermission>
		</EditorCard>
	)
}
