import { FilePlus2, Loader2 } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { SectionCard } from '#/components/ui/surface'
import type { CreationMode } from '#/pages/doe/lib/empty-state'

interface DoeEmptyStateProps {
	mode: CreationMode
	models: Array<{ id: string; name: string }>
	selectedModelId: string
	isLoading: boolean
	isPending: boolean
	error: string | null
	onSelectModel: (modelId: string) => void
	onCreate: () => void
	onCreateStarter: () => void
}

export function DoeEmptyState({
	mode,
	models,
	selectedModelId,
	isLoading,
	isPending,
	error,
	onSelectModel,
	onCreate,
	onCreateStarter,
}: DoeEmptyStateProps) {
	return (
		<SectionCard className="flex flex-col items-center gap-4 p-10 text-center">
			<FilePlus2 className="size-8 text-muted-foreground" />
			<div>
				<p className="font-semibold">Ce projet n'a pas encore de DOE</p>
				<p className="mt-1 text-sm text-muted-foreground">
					Partez d'un modèle : vous pourrez cocher les thèmes et piocher dans le
					catalogue ensuite.
				</p>
			</div>
			{isLoading ? (
				<Loader2 className="size-5 animate-spin text-muted-foreground" />
			) : (
				<RequirePermission permission="MANAGE_PLANNING">
					{mode === 'pick-model' ? (
						<div className="flex flex-wrap items-center justify-center gap-3">
							<Select value={selectedModelId} onValueChange={onSelectModel}>
								<SelectTrigger aria-label="Modèle de DOE" className="w-64">
									<SelectValue placeholder="Choisir un modèle" />
								</SelectTrigger>
								<SelectContent>
									{models.map((model) => (
										<SelectItem key={model.id} value={model.id}>
											{model.name}
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
					) : (
						<div className="flex flex-col items-center gap-2">
							<p className="text-sm text-muted-foreground">
								Aucun modèle de DOE dans votre organisation.
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
			{error ? <p className="text-sm text-destructive">{error}</p> : null}
		</SectionCard>
	)
}
