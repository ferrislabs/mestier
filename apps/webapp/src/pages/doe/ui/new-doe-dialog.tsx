import { Check, ChevronsUpDown, Loader2 } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Command,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
} from '#/components/ui/command'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Field } from '#/components/ui/field'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { cn } from '#/lib/utils'
import type { CreationMode } from '#/pages/doe/lib/empty-state'
import { canCreateDoe } from '#/pages/doe/lib/new-doe'

export interface NewDoeDialogProps {
	open: boolean
	mode: CreationMode
	projects: Array<{ id: string; name: string }>
	models: Array<{ id: string; name: string }>
	selectedProjectId: string
	selectedModelId: string
	isLoading: boolean
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onSelectProject: (projectId: string) => void
	onSelectModel: (modelId: string) => void
	onCreate: () => void
	onCreateStarter: () => void
}

export function NewDoeDialog({
	open,
	mode,
	projects,
	models,
	selectedProjectId,
	selectedModelId,
	isLoading,
	isPending,
	error,
	onOpenChange,
	onSelectProject,
	onSelectModel,
	onCreate,
	onCreateStarter,
}: NewDoeDialogProps) {
	const [pickerOpen, setPickerOpen] = useState(false)
	const selectedProject =
		projects.find((project) => project.id === selectedProjectId) ?? null

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-w-lg">
				<DialogHeader>
					<DialogTitle>Nouveau DOE</DialogTitle>
					<DialogDescription>
						Choisissez le projet et le modèle : les thèmes du modèle sont copiés
						dans le DOE et restent modifiables.
					</DialogDescription>
				</DialogHeader>
				{isLoading ? (
					<div className="flex items-center justify-center gap-2 p-6 text-sm text-muted-foreground">
						<Loader2 className="size-4 animate-spin" />
						Chargement…
					</div>
				) : (
					<div className="flex flex-col gap-4">
						<Field label="Projet" htmlFor="new-doe-project">
							<Popover open={pickerOpen} onOpenChange={setPickerOpen}>
								<PopoverTrigger asChild>
									<Button
										id="new-doe-project"
										type="button"
										variant="outline"
										role="combobox"
										aria-expanded={pickerOpen}
										className="w-full justify-between font-normal"
									>
										<span
											className={cn(
												'truncate',
												selectedProject === null && 'text-muted-foreground',
											)}
										>
											{selectedProject
												? selectedProject.name
												: projects.length === 0
													? 'Aucun projet sans DOE'
													: 'Choisir un projet…'}
										</span>
										<ChevronsUpDown className="size-4 shrink-0 opacity-50" />
									</Button>
								</PopoverTrigger>
								<PopoverContent align="start" className="w-96 p-0">
									<Command>
										<CommandInput
											autoFocus
											placeholder="Rechercher un projet…"
										/>
										<CommandList>
											<CommandEmpty>
												<p className="px-2 py-4 text-center text-sm text-muted-foreground">
													{projects.length === 0
														? 'Tous les projets ont déjà un DOE'
														: 'Aucun résultat'}
												</p>
											</CommandEmpty>
											<CommandGroup>
												{projects.map((project) => (
													<CommandItem
														key={project.id}
														value={project.name}
														onSelect={() => {
															onSelectProject(project.id)
															setPickerOpen(false)
														}}
													>
														<Check
															className={cn(
																'size-4',
																project.id === selectedProjectId
																	? 'opacity-100'
																	: 'opacity-0',
															)}
														/>
														{project.name}
													</CommandItem>
												))}
											</CommandGroup>
										</CommandList>
									</Command>
								</PopoverContent>
							</Popover>
						</Field>

						{mode === 'pick-model' ? (
							<Field label="Modèle" htmlFor="new-doe-model">
								<Select value={selectedModelId} onValueChange={onSelectModel}>
									<SelectTrigger id="new-doe-model" aria-label="Modèle de DOE">
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
							</Field>
						) : (
							<div className="flex flex-col items-start gap-2">
								<p className="text-sm text-muted-foreground">
									Aucun modèle de DOE dans votre organisation.
								</p>
								<Button
									type="button"
									variant="outline"
									disabled={isPending}
									onClick={onCreateStarter}
								>
									{isPending ? <Loader2 className="animate-spin" /> : null}
									Créer le modèle Paysagiste
								</Button>
							</div>
						)}
					</div>
				)}
				{error ? <p className="text-sm text-destructive">{error}</p> : null}
				<DialogFooter>
					<Button
						type="button"
						variant="outline"
						onClick={() => onOpenChange(false)}
					>
						Annuler
					</Button>
					<Button
						type="button"
						disabled={
							!canCreateDoe(selectedProjectId, selectedModelId) || isPending
						}
						onClick={onCreate}
					>
						{isPending ? <Loader2 className="animate-spin" /> : null}
						Créer
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
