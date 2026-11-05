import { Plus, Sparkles } from 'lucide-react'
import type { ReactNode } from 'react'
import { EmptyState } from '#/components/empty-state'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import type { DoeModel } from '#/hooks/use-doe-models'
import {
	isModelStatus,
	type ModelStatus,
} from '#/pages/doe-models/lib/models-filter'
import { ModelsTable } from '#/pages/doe-models/ui/models-table'
import { Page, PageHeader } from '#/pages/projects/ui/page'

export interface DoeModelsPageProps {
	embedded?: boolean
	organizationSlug: string
	status: ModelStatus
	counts: { active: number; archived: number }
	models: readonly DoeModel[]
	notice: string
	isLoading: boolean
	error: string | null
	onStatusChange: (status: ModelStatus) => void
	onNew: () => void
	onStarters: () => void
	onArchive: (model: DoeModel) => void
	onRestore: (model: DoeModel) => void
	onCreateDoe: (model: DoeModel) => void
}

export function DoeModelsPage({
	embedded = false,
	organizationSlug,
	status,
	counts,
	models,
	notice,
	isLoading,
	error,
	onStatusChange,
	onNew,
	onStarters,
	onArchive,
	onRestore,
	onCreateDoe,
}: DoeModelsPageProps) {
	const Wrapper = embedded ? EmbeddedFrame : Page
	const Header = embedded ? EmbeddedHeader : PageHeader

	return (
		<Wrapper>
			<Header
				title="Modèles de DOE"
				description="Des listes de thèmes préréglées, communes à l’organisation, pour créer le DOE d’un projet."
				actions={
					<RequirePermission permission="MANAGE_REFERENCE">
						<Button variant="outline" onClick={onStarters}>
							<Sparkles aria-hidden />
							Créer depuis un modèle de départ
						</Button>
						<Button onClick={onNew}>
							<Plus aria-hidden />
							Nouveau modèle
						</Button>
					</RequirePermission>
				}
			/>
			{error ? (
				<div
					role="alert"
					className="mb-3 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive"
				>
					{error}
				</div>
			) : null}
			<div className="mb-3 flex flex-wrap items-center justify-between gap-3">
				<ToggleGroup
					type="single"
					variant="outline"
					size="sm"
					value={status}
					onValueChange={(value) => {
						if (isModelStatus(value)) onStatusChange(value)
					}}
					aria-label="Filtrer par état"
				>
					<ToggleGroupItem value="actifs">
						Actifs ({counts.active})
					</ToggleGroupItem>
					<ToggleGroupItem value="archives">
						Archivés ({counts.archived})
					</ToggleGroupItem>
				</ToggleGroup>
				<output className="text-[13px] text-muted-foreground">{notice}</output>
			</div>
			{isLoading ? (
				<div className="flex min-h-48 items-center justify-center rounded-lg border bg-card p-8 text-sm text-muted-foreground">
					Chargement des modèles…
				</div>
			) : models.length === 0 ? (
				status === 'actifs' ? (
					<EmptyState
						title="Aucun modèle de DOE actif"
						hint="Crée un modèle, ou pars du modèle de départ Paysagiste."
					>
						<RequirePermission permission="MANAGE_REFERENCE">
							<div className="flex flex-wrap justify-center gap-2">
								<Button variant="outline" onClick={onStarters}>
									<Sparkles aria-hidden />
									Créer depuis un modèle de départ
								</Button>
								<Button onClick={onNew}>
									<Plus aria-hidden />
									Nouveau modèle
								</Button>
							</div>
						</RequirePermission>
					</EmptyState>
				) : (
					<EmptyState
						title="Aucun modèle archivé"
						hint="Les modèles que tu archives apparaissent ici et peuvent être restaurés."
					/>
				)
			) : (
				<ModelsTable
					organizationSlug={organizationSlug}
					models={models}
					onArchive={onArchive}
					onRestore={onRestore}
					onCreateDoe={onCreateDoe}
				/>
			)}
		</Wrapper>
	)
}

function EmbeddedFrame({ children }: { children: ReactNode }) {
	return <div>{children}</div>
}

function EmbeddedHeader({
	title,
	description,
	actions,
}: {
	title: string
	description?: string
	actions?: ReactNode
}) {
	return (
		<header className="mb-4 flex flex-wrap items-end justify-between gap-4">
			<div className="min-w-0">
				<h2 className="font-semibold text-lg tracking-tight">{title}</h2>
				{description && (
					<p className="mt-1 text-muted-foreground text-sm">{description}</p>
				)}
			</div>
			{actions && <div className="flex items-center gap-2">{actions}</div>}
		</header>
	)
}
