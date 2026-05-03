import { useForm } from '@tanstack/react-form'
import { useNavigate } from '@tanstack/react-router'
import { useAuth } from 'react-oidc-context'
import { useCreateOrganization } from '#/hooks/use-organizations'
import { OnboardingUI } from '#/pages/onboarding/ui/onboarding-ui'

export function OnboardingFeature() {
	const navigate = useNavigate()
	const auth = useAuth()
	const { mutateAsync, isPending, error } = useCreateOrganization()

	const form = useForm({
		defaultValues: { name: '', slug: '' },
		onSubmit: async ({ value }) => {
			await mutateAsync({ body: { name: value.name, slug: value.slug } })
			await navigate({ to: '/' })
		},
	})

	const handleNameChange = (name: string) => {
		const prevName = form.getFieldValue('name')
		const prevSlug = form.getFieldValue('slug')
		form.setFieldValue('name', name)
		if (!prevSlug || prevSlug === nameToSlug(prevName)) {
			form.setFieldValue('slug', nameToSlug(name))
		}
	}

	return (
		<form.Subscribe selector={(s) => s.values}>
			{(values) => (
				<OnboardingUI
					name={values.name}
					slug={values.slug}
					globalError={error?.message ?? null}
					isPending={isPending}
					onNameChange={handleNameChange}
					onSlugChange={(v) => form.setFieldValue('slug', v)}
					onSubmit={() => void form.handleSubmit()}
					onLogout={() => void auth.signoutRedirect()}
				/>
			)}
		</form.Subscribe>
	)
}

function nameToSlug(name: string): string {
	return name
		.toLowerCase()
		.normalize('NFD')
		.replace(/[\u0300-\u036f]/g, '')
		.replace(/\s+/g, '-')
		.replace(/[^a-z0-9-]/g, '')
		.replace(/-{2,}/g, '-')
		.replace(/^-|-$/g, '')
}
