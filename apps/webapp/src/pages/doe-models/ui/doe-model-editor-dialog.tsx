import { ArrowDown, ArrowUp, Loader2, Plus, Trash2 } from 'lucide-react'
import { type ReactNode, useState } from 'react'
import { TextField } from '#/components/reference-table'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { Checkbox } from '#/components/ui/checkbox'
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
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { Switch } from '#/components/ui/switch'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'
import { Textarea } from '#/components/ui/textarea'
import { cn } from '#/lib/utils'
import { moveItem } from '#/pages/doe/lib/sections'
import {
	KIND_LABELS,
	MAKER_LABEL_TEXT,
	type MakerLabel,
	type PhotosPerPage,
	SECTION_KINDS,
	type SectionKind,
} from '#/pages/doe/types'
import {
	canSubmitModel,
	MAX_BLANK_PAGES,
	MIN_BLANK_PAGES,
} from '#/pages/doe-models/lib/model-form'
import type { ModelDraft, ModelSectionDraft } from '#/pages/doe-models/types'

interface DoeModelEditorDialogProps {
	open: boolean
	draft: ModelDraft
	isPending: boolean
	error: string | null
	styleEditor: ReactNode
	onStyleToggle: (enabled: boolean) => void
	onOpenChange: (open: boolean) => void
	onChange: (patch: Partial<ModelDraft>) => void
	onAddSection: (title: string, kind: SectionKind) => void
	onSubmit: () => void
}

export function DoeModelEditorDialog({
	open,
	draft,
	isPending,
	error,
	styleEditor,
	onStyleToggle,
	onOpenChange,
	onChange,
	onAddSection,
	onSubmit,
}: DoeModelEditorDialogProps) {
	const [newTitle, setNewTitle] = useState('')
	const [tab, setTab] = useState('sections')
	const [newKind, setNewKind] = useState<SectionKind>('supplies')

	const patchSection = (key: string, patch: Partial<ModelSectionDraft>) =>
		onChange({
			sections: draft.sections.map((section) =>
				section.key === key ? { ...section, ...patch } : section,
			),
		})

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent
				className={cn(
					'flex max-h-[90vh] flex-col gap-0 overflow-hidden',
					tab === 'style' ? 'sm:max-w-6xl' : 'sm:max-w-3xl',
				)}
			>
				<DialogHeader className="border-b pb-4">
					<DialogTitle>
						{draft.id ? 'Modifier le modèle' : 'Nouveau modèle de DOE'}
					</DialogTitle>
					<DialogDescription>
						Les thèmes décochés sont créés désactivés dans le DOE, l'utilisateur
						peut les activer.
					</DialogDescription>
				</DialogHeader>

				<div className="flex-1 space-y-4 overflow-y-auto py-4">
					<TextField
						label="Nom"
						value={draft.name}
						onChange={(name) => onChange({ name })}
						placeholder="Paysagiste, Gros œuvre…"
					/>
					<div className="space-y-1">
						<Label htmlFor="doe-model-description">Description</Label>
						<Textarea
							id="doe-model-description"
							value={draft.description}
							onChange={(event) =>
								onChange({ description: event.target.value })
							}
						/>
					</div>

					<Tabs value={tab} onValueChange={setTab}>
						<TabsList>
							<TabsTrigger value="sections">Sections</TabsTrigger>
							<TabsTrigger value="style">Style</TabsTrigger>
						</TabsList>
						<TabsContent value="sections" className="pt-4">
							<div className="space-y-2">
								<Label>Thèmes</Label>
								{draft.sections.length === 0 ? (
									<p className="text-sm text-muted-foreground">
										Aucun thème pour l'instant.
									</p>
								) : (
									<ul className="divide-y rounded-md border">
										{draft.sections.map((section, index) => (
											<li
												key={section.key}
												className="flex flex-wrap items-center gap-3 px-3 py-2"
											>
												<Checkbox
													checked={section.enabled}
													aria-label={`Cochée par défaut : ${section.title || `thème ${index + 1}`}`}
													onCheckedChange={(checked) =>
														patchSection(section.key, {
															enabled: checked === true,
														})
													}
												/>
												<Input
													value={section.title}
													aria-label={`Titre du thème ${index + 1}`}
													className="min-w-40 flex-1"
													onChange={(event) =>
														patchSection(section.key, {
															title: event.target.value,
														})
													}
												/>
												<span className="w-28 text-sm text-muted-foreground">
													{KIND_LABELS[section.kind]}
												</span>
												<SectionSettings
													section={section}
													onChange={(patch) => patchSection(section.key, patch)}
												/>
												<div className="ml-auto flex items-center">
													<Button
														type="button"
														variant="ghost"
														size="icon-sm"
														disabled={index === 0}
														onClick={() =>
															onChange({
																sections: moveItem(draft.sections, index, -1),
															})
														}
													>
														<ArrowUp />
														<span className="sr-only">
															Monter le thème {index + 1}
														</span>
													</Button>
													<Button
														type="button"
														variant="ghost"
														size="icon-sm"
														disabled={index === draft.sections.length - 1}
														onClick={() =>
															onChange({
																sections: moveItem(draft.sections, index, 1),
															})
														}
													>
														<ArrowDown />
														<span className="sr-only">
															Descendre le thème {index + 1}
														</span>
													</Button>
													<Button
														type="button"
														variant="ghost"
														size="icon-sm"
														onClick={() =>
															onChange({
																sections: draft.sections.filter(
																	(entry) => entry.key !== section.key,
																),
															})
														}
													>
														<Trash2 />
														<span className="sr-only">
															Supprimer le thème {index + 1}
														</span>
													</Button>
												</div>
											</li>
										))}
									</ul>
								)}

								<div className="flex flex-wrap items-end gap-3 pt-2">
									<div className="flex flex-col gap-1">
										<Label htmlFor="doe-model-new-title">
											Titre du nouveau thème
										</Label>
										<Input
											id="doe-model-new-title"
											value={newTitle}
											className="w-64"
											onChange={(event) => setNewTitle(event.target.value)}
										/>
									</div>
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
									<Button
										type="button"
										variant="outline"
										disabled={newTitle.trim() === ''}
										onClick={() => {
											onAddSection(newTitle, newKind)
											setNewTitle('')
										}}
									>
										<Plus />
										Ajouter un thème
									</Button>
								</div>
							</div>
						</TabsContent>
						<TabsContent value="style" className="flex flex-col gap-4 pt-4">
							<div className="flex items-center gap-3">
								<Switch
									aria-label="Style propre à ce modèle"
									checked={draft.style !== null}
									onCheckedChange={onStyleToggle}
								/>
								<span className="text-sm">Style propre à ce modèle</span>
							</div>
							{draft.style === null ? (
								<p className="text-sm text-muted-foreground">
									Le style de l'organisation s'applique aux DOE créés depuis ce
									modèle.
								</p>
							) : (
								styleEditor
							)}
						</TabsContent>
					</Tabs>

					{error ? <p className="text-sm text-destructive">{error}</p> : null}
				</div>

				<DialogFooter className="border-t pt-4">
					<Button variant="outline" onClick={() => onOpenChange(false)}>
						Annuler
					</Button>
					<RequirePermission permission="MANAGE_REFERENCE">
						<Button
							disabled={!canSubmitModel(draft) || isPending}
							onClick={onSubmit}
						>
							{isPending ? <Loader2 className="animate-spin" /> : null}
							{draft.id ? 'Enregistrer' : 'Créer'}
						</Button>
					</RequirePermission>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}

function SectionSettings({
	section,
	onChange,
}: {
	section: ModelSectionDraft
	onChange: (patch: Partial<ModelSectionDraft>) => void
}) {
	switch (section.kind) {
		case 'supplies':
			return (
				<Select
					value={section.makerLabel}
					onValueChange={(value) =>
						onChange({ makerLabel: value as MakerLabel })
					}
				>
					<SelectTrigger
						aria-label={`Libellé du fabricant de ${section.title}`}
						size="sm"
						className="w-32"
					>
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="manufacturer">
							{MAKER_LABEL_TEXT.manufacturer}
						</SelectItem>
						<SelectItem value="nursery">{MAKER_LABEL_TEXT.nursery}</SelectItem>
					</SelectContent>
				</Select>
			)
		case 'photos':
			return (
				<Select
					value={section.perPage}
					onValueChange={(value) =>
						onChange({ perPage: value as PhotosPerPage })
					}
				>
					<SelectTrigger
						aria-label={`Photos par page de ${section.title}`}
						size="sm"
						className="w-32"
					>
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="one">1 par page</SelectItem>
						<SelectItem value="two">2 par page</SelectItem>
						<SelectItem value="four">4 par page</SelectItem>
					</SelectContent>
				</Select>
			)
		case 'blank_pages':
			return (
				<Input
					type="number"
					min={MIN_BLANK_PAGES}
					max={MAX_BLANK_PAGES}
					value={section.count}
					aria-label={`Nombre de pages libres de ${section.title}`}
					className="w-20"
					onChange={(event) =>
						onChange({ count: Number(event.target.value) || MIN_BLANK_PAGES })
					}
				/>
			)
		case 'plans':
			return null
	}
}
