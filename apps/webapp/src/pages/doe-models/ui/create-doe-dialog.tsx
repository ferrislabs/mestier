import { Link } from '@tanstack/react-router'
import { Loader2 } from 'lucide-react'
import { useId, useState } from 'react'
import { FormField } from '#/components/form-field'
import { Button, buttonVariants } from '#/components/ui/button'
import {
	Dialog,
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { buildOrgPath } from '#/modules/org-path'

export interface DoeTargetOption {
	id: string
	label: string
}

export function CreateDoeDialog({
	organizationSlug,
	modelName,
	open,
	targets,
	noTargetMessage,
	isPending,
	error,
	onOpenChange,
	onCreate,
}: {
	organizationSlug: string
	modelName: string
	open: boolean
	targets: readonly DoeTargetOption[]
	noTargetMessage: string
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onCreate: (projectId: string) => void
}) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="sm:max-w-md">
				{open && (
					<CreateDoeForm
						organizationSlug={organizationSlug}
						modelName={modelName}
						targets={targets}
						noTargetMessage={noTargetMessage}
						isPending={isPending}
						error={error}
						onClose={() => onOpenChange(false)}
						onCreate={onCreate}
					/>
				)}
			</DialogContent>
		</Dialog>
	)
}

function CreateDoeForm({
	organizationSlug,
	modelName,
	targets,
	noTargetMessage,
	isPending,
	error,
	onClose,
	onCreate,
}: {
	organizationSlug: string
	modelName: string
	targets: readonly DoeTargetOption[]
	noTargetMessage: string
	isPending: boolean
	error: string | null
	onClose: () => void
	onCreate: (projectId: string) => void
}) {
	const baseId = useId()
	const [picked, setPicked] = useState<string | null>(null)
	const projectId = targets.some((project) => project.id === picked)
		? picked
		: (targets[0]?.id ?? null)

	return (
		<>
			<DialogHeader>
				<DialogTitle>Créer un DOE avec ce modèle</DialogTitle>
				<DialogDescription>
					Les thèmes du modèle « {modelName} » sont copiés dans le DOE et
					restent modifiables. Un projet reçoit au plus un DOE.
				</DialogDescription>
			</DialogHeader>
			{targets.length === 0 ? (
				<div className="grid gap-3 rounded-md border border-dashed p-4 text-[13px]">
					<p>{noTargetMessage}</p>
					<Link
						to={buildOrgPath(organizationSlug, '/planification/projects/new')}
						onClick={onClose}
						className={buttonVariants({
							variant: 'outline',
							size: 'sm',
							className: 'w-fit',
						})}
					>
						Nouveau projet
					</Link>
				</div>
			) : (
				<FormField
					id={`${baseId}-project`}
					label="Projet"
					hint="Seuls les projets actifs sans DOE sont proposés."
				>
					<Select
						value={projectId ?? undefined}
						onValueChange={(value) => setPicked(value)}
					>
						<SelectTrigger id={`${baseId}-project`} className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{targets.map((project) => (
								<SelectItem key={project.id} value={project.id}>
									{project.label}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</FormField>
			)}
			{error && (
				<p role="alert" className="text-[13px] text-state-alert">
					{error}
				</p>
			)}
			<DialogFooter>
				<DialogClose asChild>
					<Button type="button" variant="outline">
						Annuler
					</Button>
				</DialogClose>
				<Button
					type="button"
					disabled={!projectId || isPending}
					onClick={() => {
						if (projectId) onCreate(projectId)
					}}
				>
					{isPending ? <Loader2 className="animate-spin" /> : null}
					Créer le DOE
				</Button>
			</DialogFooter>
		</>
	)
}
