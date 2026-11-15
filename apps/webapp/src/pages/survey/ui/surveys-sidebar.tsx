import { Link } from '@tanstack/react-router'
import { ClipboardList, Plus, Search } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { cn } from '#/lib/utils'
import { SURVEY_STATUS, VISIBILITY } from '#/pages/survey/lib/survey-format'
import type { Survey } from '#/pages/survey/types'

export function SurveysSidebar({
	surveys,
	selectedId,
	search,
	canManage,
	isLoading,
	error,
	itemPath,
	onSearch,
	onCreate,
}: {
	surveys: readonly Survey[]
	selectedId: string | null
	search: string
	canManage: boolean
	isLoading: boolean
	error: string | null
	itemPath: string
	onSearch: (value: string) => void
	onCreate: () => void
}) {
	return (
		<aside
			aria-label="Questionnaires"
			className="flex min-h-0 flex-col border-b bg-card lg:border-r lg:border-b-0"
		>
			<div className="grid gap-2 border-b p-3">
				<div className="flex items-center justify-between gap-2">
					<h2 className="font-semibold text-sm">Questionnaires</h2>
					{canManage ? (
						<Button type="button" size="sm" onClick={onCreate}>
							<Plus aria-hidden />
							Nouveau
						</Button>
					) : null}
				</div>
				<div className="relative">
					<Search
						className="-translate-y-1/2 pointer-events-none absolute top-1/2 left-2.5 size-4 text-muted-foreground"
						aria-hidden
					/>
					<Input
						type="search"
						aria-label="Rechercher un questionnaire"
						placeholder="Rechercher…"
						value={search}
						className="h-9 pl-8 text-sm"
						onChange={(event) => onSearch(event.target.value)}
					/>
				</div>
			</div>
			<nav
				aria-label="Liste des questionnaires"
				className="min-h-0 max-h-72 flex-1 overflow-y-auto lg:max-h-none"
			>
				{error ? (
					<p role="alert" className="p-3 text-destructive text-sm">
						{error}
					</p>
				) : isLoading ? (
					<p className="p-3 text-muted-foreground text-sm">Chargement…</p>
				) : surveys.length === 0 ? (
					<p className="p-3 text-muted-foreground text-sm">
						{search ? 'Aucun résultat.' : 'Aucun questionnaire pour le moment.'}
					</p>
				) : (
					<ul>
						{surveys.map((survey) => {
							const selected = survey.id === selectedId
							return (
								<li key={survey.id}>
									<Link
										to={itemPath}
										params={{ surveyId: survey.id }}
										aria-current={selected ? 'page' : undefined}
										className={cn(
											'relative flex items-center gap-3 border-b px-3 py-2.5 outline-none transition-colors hover:bg-accent/60 focus-visible:bg-accent',
											selected && 'bg-accent',
										)}
									>
										{selected ? (
											<span
												aria-hidden
												className="absolute inset-y-0 left-0 w-0.5 bg-primary"
											/>
										) : null}
										<span
											aria-hidden
											className="flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground"
										>
											<ClipboardList className="size-4" />
										</span>
										<span className="grid min-w-0 flex-1 gap-0.5">
											<span className="truncate font-medium text-sm">
												{survey.title}
											</span>
											<span className="truncate text-muted-foreground text-xs">
												{SURVEY_STATUS[survey.status].label}
												{' · '}
												{VISIBILITY[survey.visibility].label}
											</span>
										</span>
									</Link>
								</li>
							)
						})}
					</ul>
				)}
			</nav>
		</aside>
	)
}
