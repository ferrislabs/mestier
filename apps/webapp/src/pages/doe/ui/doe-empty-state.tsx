import { Link } from '@tanstack/react-router'
import { FilePlus2, Loader2 } from 'lucide-react'
import { useId } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { buildOrgPath } from '#/modules/org-path'
import type { CreationMode } from '#/pages/doe/lib/empty-state'

interface DoeEmptyStateProps {
	organizationSlug: string
	mode: CreationMode
	models: Array<{ id: string; name: string; description?: string | null }>
	selectedModelId: string
	hasCustomer: boolean
	isLoading: boolean
	isPending: boolean
	error: string | null
	onSelectModel: (modelId: string) => void
	onCreate: () => void
	onCreateStarter: () => void
}

export function DoeEmptyState({
	organizationSlug,
	mode,
	models,
	selectedModelId,
	hasCustomer,
	isLoading,
	isPending,
	error,
	onSelectModel,
	onCreate,
	onCreateStarter,
}: DoeEmptyStateProps) {
	const titleId = useId()
	const model = models.find((item) => item.id === selectedModelId)

	return (
		<section
			aria-labelledby={titleId}
			className="flex flex-col items-center gap-4 rounded-lg border bg-card px-6 py-10 text-center"
		>
			<FilePlus2 className="size-8 text-muted-foreground" aria-hidden />
			<div className="max-w-xl">
				<h2 id={titleId} className="font-semibold">
					Ce projet n’a pas encore de DOE
				</h2>
				<p className="mt-1 text-muted-foreground text-sm">
					Pars d’un modèle : tu pourras ensuite cocher les thèmes à inclure et
					piocher dans les fiches techniques.
				</p>
			</div>
			{isLoading ? (
				<Loader2
					className="size-5 animate-spin text-muted-foreground"
					aria-hidden
				/>
			) : (
				<RequirePermission permission="MANAGE_PLANNING">
					{mode === 'pick-model' ? (
						<div className="grid justify-items-center gap-2">
							<div className="flex flex-wrap items-center justify-center gap-3">
								<Select value={selectedModelId} onValueChange={onSelectModel}>
									<SelectTrigger aria-label="Modèle de DOE" className="w-64">
										<SelectValue placeholder="Choisir un modèle" />
									</SelectTrigger>
									<SelectContent>
										{models.map((item) => (
											<SelectItem key={item.id} value={item.id}>
												{item.name}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
								<Button
									type="button"
									disabled={selectedModelId === '' || isPending}
									onClick={onCreate}
								>
									{isPending ? <Loader2 className="animate-spin" /> : null}
									Créer le DOE
								</Button>
							</div>
							{model?.description ? (
								<p className="text-[13px] text-muted-foreground">
									{model.description}
								</p>
							) : null}
						</div>
					) : (
						<div className="grid justify-items-center gap-2">
							<p className="text-muted-foreground text-sm">
								Aucun modèle de DOE dans ton organisation.
							</p>
							<Button
								type="button"
								disabled={isPending}
								onClick={onCreateStarter}
							>
								{isPending ? <Loader2 className="animate-spin" /> : null}
								Créer le modèle Paysagiste
							</Button>
						</div>
					)}
				</RequirePermission>
			)}
			{error ? (
				<p role="alert" className="text-[13px] text-state-alert">
					{error}
				</p>
			) : null}
			<div className="max-w-xl text-[13px] text-muted-foreground">
				<p>
					Les intervenants sont préremplis : la maîtrise d’ouvrage depuis le
					client du projet, l’entreprise depuis ton organisation.
					{!hasCustomer &&
						' Ce projet n’a pas de client : la maîtrise d’ouvrage restera à compléter.'}
				</p>
				<p className="mt-2">
					<Link
						to={buildOrgPath(organizationSlug, '/planification/doe-models')}
						className="rounded-sm font-medium text-primary underline underline-offset-4 outline-none hover:no-underline focus-visible:ring-2 focus-visible:ring-ring"
					>
						Gérer les modèles de DOE
					</Link>
				</p>
			</div>
		</section>
	)
}
