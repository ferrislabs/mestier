import { ArrowDown, ArrowUp, Plus, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import { Checkbox } from '#/components/ui/checkbox'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import { numberSections } from '#/pages/doe/lib/sections'
import {
	type DoeSection,
	KIND_LABELS,
	SECTION_KINDS,
	type SectionKind,
} from '#/pages/doe/types'

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
	const [newTitle, setNewTitle] = useState('')
	const [newKind, setNewKind] = useState<SectionKind>('supplies')
	const numbered = numberSections(sections)

	return (
		<SectionCard>
			<SectionHeader
				title="Plan du DOE"
				description="Cochez les thèmes à inclure. La numérotation ne compte que les thèmes cochés."
			/>
			<ul className="divide-y">
				{numbered.map(({ section, number }, index) => (
					<li key={section.id} className="flex items-center gap-3 px-5 py-3">
						<Checkbox
							checked={section.enabled}
							aria-label={`Inclure ${section.title}`}
							onCheckedChange={(checked) =>
								onToggle(section.id, checked === true)
							}
						/>
						<span className="w-6 text-center text-sm tabular-nums text-muted-foreground">
							{number ?? '—'}
						</span>
						<Input
							value={section.title}
							aria-label={`Titre du thème ${index + 1}`}
							className="max-w-sm"
							onChange={(event) => onRename(section.id, event.target.value)}
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
								<ArrowUp />
								<span className="sr-only">Monter {section.title}</span>
							</Button>
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								disabled={index === sections.length - 1}
								onClick={() => onMove(index, 1)}
							>
								<ArrowDown />
								<span className="sr-only">Descendre {section.title}</span>
							</Button>
							{removableSectionIds.has(section.id) ? (
								<Button
									type="button"
									variant="ghost"
									size="icon-sm"
									onClick={() => onRemove(section.id)}
								>
									<Trash2 />
									<span className="sr-only">Supprimer {section.title}</span>
								</Button>
							) : null}
						</div>
					</li>
				))}
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
					<div className="flex flex-col gap-1">
						<label
							htmlFor="doe-new-section-title"
							className="text-sm font-medium"
						>
							Titre du nouveau thème
						</label>
						<Input
							id="doe-new-section-title"
							value={newTitle}
							className="w-64"
							onChange={(event) => setNewTitle(event.target.value)}
						/>
					</div>
					<div className="flex flex-col gap-1">
						<span className="text-sm font-medium">Type</span>
						<Select
							value={newKind}
							onValueChange={(value) => setNewKind(value as SectionKind)}
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
						<Plus />
						Ajouter un thème
					</Button>
				</form>
			</RequirePermission>
		</SectionCard>
	)
}
