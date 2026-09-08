import { useForm } from '@tanstack/react-form'
import { useNavigate } from '@tanstack/react-router'
import { Loader2 } from 'lucide-react'

import { Button } from '#/components/ui/button'
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
import { useCreateOrganization } from '#/hooks/use-organizations'
import { buildOrgPath } from '#/modules/org-path'
import {
	normalizeSlugForPayload,
	normalizeSlugInput,
	slugFromName,
} from '#/modules/org-slug'

interface CreateOrganizationDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
}

/**
 * Creates an additional organization from inside the application — the
 * onboarding screen only ever shows up for an account that has none.
 *
 * Mount it only while open (`{open ? <CreateOrganizationDialog … /> : null}`):
 * the form state is held here, so unmounting is what resets it between two
 * attempts.
 */
export function CreateOrganizationDialog({
	open,
	onOpenChange,
}: CreateOrganizationDialogProps) {
	const navigate = useNavigate()
	const { mutateAsync, isPending, error } = useCreateOrganization()

	const form = useForm({
		defaultValues: { name: '', slug: '' },
		onSubmit: async ({ value }) => {
			// A rejection is already surfaced by the mutation's `error`, and the
			// dialog stays open on it — swallowing it here keeps the submit
			// handler from rethrowing into an unhandled rejection.
			const created = await mutateAsync({
				body: {
					name: value.name.trim(),
					slug: normalizeSlugForPayload(value.slug),
				},
			}).catch(() => null)
			if (!created) return

			// `useCreateOrganization` waits for the organization list to be
			// refetched before resolving: the tenant comes from the URL, and the
			// layout refuses a slug the list does not carry yet.
			onOpenChange(false)
			await navigate({ to: buildOrgPath(created.data.slug, '/') })
		},
	})

	// The slug follows the name until it is edited by hand.
	const handleNameChange = (name: string) => {
		const previousName = form.getFieldValue('name')
		const previousSlug = form.getFieldValue('slug')
		form.setFieldValue('name', name)
		if (!previousSlug || previousSlug === slugFromName(previousName)) {
			form.setFieldValue('slug', slugFromName(name))
		}
	}

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Nouvelle organisation</DialogTitle>
					<DialogDescription>
						Un espace de travail séparé, avec ses propres clients, devis et
						équipe.
					</DialogDescription>
				</DialogHeader>

				<form
					id="create-organization"
					onSubmit={(event) => {
						event.preventDefault()
						void form.handleSubmit()
					}}
					className="flex flex-col gap-4"
				>
					<form.Subscribe selector={(state) => state.values}>
						{(values) => (
							<>
								<div className="flex flex-col gap-1.5">
									<Label htmlFor="new-organization-name">
										Nom de l'organisation
									</Label>
									<Input
										id="new-organization-name"
										placeholder="Entreprise Dupont"
										value={values.name}
										onChange={(event) => handleNameChange(event.target.value)}
										disabled={isPending}
									/>
								</div>

								<div className="flex flex-col gap-1.5">
									<div className="flex items-center justify-between">
										<Label htmlFor="new-organization-slug">
											Identifiant (slug)
										</Label>
										<span className="text-[10px] text-muted-foreground">
											Utilisé dans les URLs
										</span>
									</div>
									<Input
										id="new-organization-slug"
										placeholder="entreprise-dupont"
										value={values.slug}
										onChange={(event) =>
											form.setFieldValue(
												'slug',
												normalizeSlugInput(event.target.value),
											)
										}
										disabled={isPending}
										className="font-mono text-sm"
									/>
								</div>

								{error ? (
									<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-3 py-2 text-sm text-destructive">
										{error.message}
									</p>
								) : null}

								<DialogFooter>
									<Button
										type="button"
										variant="ghost"
										onClick={() => onOpenChange(false)}
										disabled={isPending}
									>
										Annuler
									</Button>
									<Button
										type="submit"
										disabled={isPending || !values.name.trim() || !values.slug}
									>
										{isPending ? (
											<Loader2 className="size-4 animate-spin" />
										) : null}
										Créer l'organisation
									</Button>
								</DialogFooter>
							</>
						)}
					</form.Subscribe>
				</form>
			</DialogContent>
		</Dialog>
	)
}
