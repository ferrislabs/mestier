import { Search } from 'lucide-react'
import { useId } from 'react'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'

export interface ProjectOption {
	id: string
	name: string
}

interface PhotoFiltersProps {
	q: string
	tagInput: string
	projectId: string | null
	toSort: boolean
	projects: readonly ProjectOption[]
	onQChange: (q: string) => void
	onTagInputChange: (tagInput: string) => void
	onProjectChange: (projectId: string | null) => void
	onToSortChange: (toSort: boolean) => void
}

const ALL_PROJECTS = '__all__'

export function PhotoFilters({
	q,
	tagInput,
	projectId,
	toSort,
	projects,
	onQChange,
	onTagInputChange,
	onProjectChange,
	onToSortChange,
}: PhotoFiltersProps) {
	const baseId = useId()
	return (
		<search
			aria-label="Filtres de la photothèque"
			className="flex flex-wrap items-end gap-3"
		>
			<div className="grid min-w-52 flex-1 gap-1">
				<Label htmlFor={`${baseId}-q`} className="text-muted-foreground">
					Recherche
				</Label>
				<div className="relative">
					<Search
						aria-hidden
						className="absolute top-2.5 left-2.5 size-4 text-muted-foreground"
					/>
					<Input
						id={`${baseId}-q`}
						type="search"
						value={q}
						placeholder="Légende ou étiquette"
						autoComplete="off"
						className="pl-8"
						onChange={(event) => onQChange(event.target.value)}
					/>
				</div>
			</div>
			<div className="grid min-w-44 gap-1">
				<Label htmlFor={`${baseId}-tags`} className="text-muted-foreground">
					Étiquettes
				</Label>
				<Input
					id={`${baseId}-tags`}
					value={tagInput}
					placeholder="terrasse, buis"
					autoComplete="off"
					onChange={(event) => onTagInputChange(event.target.value)}
				/>
			</div>
			<div className="grid min-w-44 gap-1">
				<Label htmlFor={`${baseId}-project`} className="text-muted-foreground">
					Projet
				</Label>
				<Select
					value={projectId ?? ALL_PROJECTS}
					onValueChange={(value) =>
						onProjectChange(value === ALL_PROJECTS ? null : value)
					}
				>
					<SelectTrigger id={`${baseId}-project`}>
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={ALL_PROJECTS}>Tous les projets</SelectItem>
						{projects.map((project) => (
							<SelectItem key={project.id} value={project.id}>
								{project.name}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</div>
			<ToggleGroup
				type="single"
				variant="outline"
				value={toSort ? 'to-sort' : ''}
				onValueChange={(value) => onToSortChange(value === 'to-sort')}
			>
				<ToggleGroupItem value="to-sort">À classer</ToggleGroupItem>
			</ToggleGroup>
		</search>
	)
}
