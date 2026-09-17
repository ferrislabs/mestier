import { FolderKanban, Search, X } from 'lucide-react'
import type { FormEvent } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { Switch } from '#/components/ui/switch'
import type { BoardFilters } from '#/pages/planification/lib/board-filters'

export interface BoardFilterOption {
	id: string
	label: string
}

export interface BoardToolbarProps {
	/** The filter state, read straight from the URL by the route. */
	filters: BoardFilters
	projects: BoardFilterOption[]
	assignees: BoardFilterOption[]
	labels: BoardFilterOption[]
	/** True while the option lists are still loading — the pickers stay usable, they just have nothing to offer yet. */
	isLoading: boolean
	/** True when at least one filter narrows the board. */
	isFiltered: boolean
	/**
	 * Whether the current filters make the listing reach past the root
	 * tasks — see `boardFiltersIncludeSubtasks`. Said out loud rather than
	 * left for the reader to notice in the counts.
	 */
	includesSubtasks: boolean
	/** One changed filter at a time; the feature merges it into the rest. */
	onChange: (patch: Partial<BoardFilters>) => void
	onClear: () => void
}

/**
 * The board's filter bar (#467) — pure presentation: props in, callbacks
 * out, no hooks and no fetching.
 *
 * The project picker is first and deliberately the widest control on the
 * row: filling the screen with one project is the question this bar exists
 * to answer, and the other three narrow within that.
 *
 * Native `<select>` rather than the styled combobox, following
 * `projects/ui/project-form-dialog.tsx`: no search inside the picker, and
 * the option lists are capped upstream (100 projects, one organization's
 * roster, one organization's labels). A searchable picker is a separate
 * change, not a half-built one here.
 */
export function BoardToolbar({
	filters,
	projects,
	assignees,
	labels,
	isLoading,
	isFiltered,
	includesSubtasks,
	onChange,
	onClear,
}: BoardToolbarProps) {
	/**
	 * The search box commits on submit rather than on every keystroke: `q`
	 * lives in the address, and one history entry per typed character would
	 * make the back button useless. The input is uncontrolled and keyed on
	 * the applied value, so moving back and forward through filter states
	 * still resets what it shows — the URL stays the single source of truth,
	 * and the half-typed word in between is nobody's state but the DOM's.
	 */
	function handleSearchSubmit(event: FormEvent<HTMLFormElement>) {
		event.preventDefault()
		const value = String(new FormData(event.currentTarget).get('q') ?? '')
		onChange({ q: value.trim() === '' ? undefined : value.trim() })
	}

	return (
		<div className="flex flex-col gap-3 rounded-md border bg-muted/20 p-3">
			<div className="flex flex-col gap-3 lg:flex-row lg:items-end">
				<div className="flex-1 space-y-1 lg:max-w-md">
					<Label
						htmlFor="board-filter-project"
						className="text-xs font-semibold"
					>
						Projet
					</Label>
					<div className="flex items-center gap-2">
						<span className="flex size-9 shrink-0 items-center justify-center rounded-md bg-brand-soft text-primary">
							<FolderKanban className="size-4" />
						</span>
						<select
							id="board-filter-project"
							className="h-10 w-full rounded-md border border-primary/30 bg-card px-3 text-sm font-medium shadow-sm"
							value={filters.project_id ?? ''}
							onChange={(event) =>
								onChange({ project_id: event.target.value || undefined })
							}
						>
							<option value="">Tous les projets</option>
							{projects.map((project) => (
								<option key={project.id} value={project.id}>
									{project.label}
								</option>
							))}
						</select>
					</div>
				</div>

				<div className="space-y-1 lg:w-52">
					<Label
						htmlFor="board-filter-assignee"
						className="text-xs font-semibold"
					>
						Assigné
					</Label>
					<select
						id="board-filter-assignee"
						className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm"
						value={filters.assignee_id ?? ''}
						onChange={(event) =>
							onChange({ assignee_id: event.target.value || undefined })
						}
					>
						<option value="">Tout le monde</option>
						{assignees.map((assignee) => (
							<option key={assignee.id} value={assignee.id}>
								{assignee.label}
							</option>
						))}
					</select>
				</div>

				<div className="space-y-1 lg:w-52">
					<Label htmlFor="board-filter-label" className="text-xs font-semibold">
						Étiquette
					</Label>
					<select
						id="board-filter-label"
						className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm"
						value={filters.label_id ?? ''}
						onChange={(event) =>
							onChange({ label_id: event.target.value || undefined })
						}
					>
						<option value="">Toutes les étiquettes</option>
						{labels.map((label) => (
							<option key={label.id} value={label.id}>
								{label.label}
							</option>
						))}
					</select>
				</div>

				<form onSubmit={handleSearchSubmit} className="space-y-1 lg:w-56">
					<Label
						htmlFor="board-filter-search"
						className="text-xs font-semibold"
					>
						Recherche
					</Label>
					<div className="flex items-center gap-2">
						<Input
							key={filters.q ?? ''}
							id="board-filter-search"
							name="q"
							type="search"
							defaultValue={filters.q ?? ''}
							placeholder="Titre de la tâche…"
						/>
						<Button type="submit" variant="outline" size="icon">
							<Search className="size-4" />
							<span className="sr-only">Rechercher</span>
						</Button>
					</div>
				</form>
			</div>

			<div className="flex flex-wrap items-center gap-x-4 gap-y-2">
				<div className="flex items-center gap-2">
					<Switch
						id="board-filter-unscheduled"
						checked={filters.unscheduled === true}
						onCheckedChange={(checked) =>
							onChange({ unscheduled: checked ? true : undefined })
						}
					/>
					<Label htmlFor="board-filter-unscheduled" className="text-sm">
						Uniquement non planifiées
					</Label>
				</div>

				{includesSubtasks ? (
					<p className="text-xs text-muted-foreground">
						Filtres actifs : les sous-tâches correspondantes sont affichées
						comme des cartes, en plus des tâches racines.
					</p>
				) : null}

				{isLoading ? (
					<p className="text-xs text-muted-foreground">
						Chargement des filtres…
					</p>
				) : null}

				{isFiltered ? (
					<Button
						type="button"
						variant="ghost"
						size="sm"
						className="ml-auto"
						onClick={onClear}
					>
						<X className="size-4" />
						Effacer les filtres
					</Button>
				) : null}
			</div>
		</div>
	)
}
