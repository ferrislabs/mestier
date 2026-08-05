import { useForm } from '@tanstack/react-form'
import { AlertCircle, Building2, Loader2, Save } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type Organization,
	useUpdateOrganization,
} from '#/hooks/use-organizations'
import type { OrganizationFormValues } from '#/pages/settings/types'
import { type FormBinding, TextField } from '#/pages/settings/ui/primitives'

export function OrganizationSection() {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-semibold">Organisation indisponible</p>
					<p className="text-sm text-muted-foreground">
						La fiche d’organisation nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<OrganizationSectionContent
			key={activeOrganization.id}
			organization={activeOrganization}
		/>
	)
}

interface OrganizationSectionContentProps {
	organization: Organization
}

function OrganizationSectionContent({
	organization,
}: OrganizationSectionContentProps) {
	const organizationId = organization.id
	const updateOrganization = useUpdateOrganization(organizationId)

	const organizationForm = useForm({
		defaultValues: {
			name: organization.name,
			slug: organization.slug,
		} satisfies OrganizationFormValues,
		onSubmit: async ({ value }) => {
			await updateOrganization.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					name: value.name.trim(),
					slug: normalizeSlugForPayload(value.slug),
				},
			})
		},
	})

	return (
		<organizationForm.Subscribe selector={(state) => state.values}>
			{(values) => {
				const form: FormBinding<OrganizationFormValues> = {
					values,
					isPending: updateOrganization.isPending,
					onChange: (patch) => {
						for (const key of Object.keys(
							patch,
						) as (keyof OrganizationFormValues)[]) {
							organizationForm.setFieldValue(key, patch[key] ?? '')
						}
					},
					onSubmit: () => void organizationForm.handleSubmit(),
				}

				const hasChanges =
					form.values.name.trim() !== organization.name ||
					form.values.slug.trim() !== organization.slug

				return (
					<SectionCard>
						<SectionHeader
							title="Organisation"
							description="Informations visibles dans l’application et utilisées pour identifier l’espace de travail."
							actions={
								<StatusBadge tone="brand">
									<Building2 className="mr-1 size-3" />
									{organization.slug}
								</StatusBadge>
							}
						/>
						<div className="grid grid-cols-1 gap-4 p-5 lg:grid-cols-[1fr_auto] lg:items-end">
							<div className="grid grid-cols-1 gap-4 md:grid-cols-2">
								<TextField
									label="Nom"
									value={form.values.name}
									onChange={(name) => form.onChange({ name })}
									placeholder="Nom de l’entreprise"
								/>
								<TextField
									label="Identifiant"
									value={form.values.slug}
									onChange={(slug) =>
										form.onChange({ slug: normalizeSlugForDisplay(slug) })
									}
									placeholder="mon-entreprise"
									className="font-mono text-sm"
								/>
							</div>
							<Button
								type="button"
								onClick={form.onSubmit}
								disabled={form.isPending || !hasChanges}
								className="gap-2"
							>
								{form.isPending ? (
									<Loader2 className="size-4 animate-spin" />
								) : (
									<Save className="size-4" />
								)}
								Enregistrer
							</Button>
						</div>
					</SectionCard>
				)
			}}
		</organizationForm.Subscribe>
	)
}

// Divergence preserved intentionally (pre-existing, identical to main): display normalization drops the trailing-dash trim that the payload normalization applies — still needs a decision.
function normalizeSlugForDisplay(value: string): string {
	return value
		.toLowerCase()
		.trim()
		.replace(/\s+/g, '-')
		.replace(/[^a-z0-9-]/g, '')
		.replace(/-{2,}/g, '-')
}

function normalizeSlugForPayload(value: string): string {
	return value
		.toLowerCase()
		.trim()
		.replace(/\s+/g, '-')
		.replace(/[^a-z0-9-]/g, '')
		.replace(/-{2,}/g, '-')
		.replace(/^-|-$/g, '')
}
