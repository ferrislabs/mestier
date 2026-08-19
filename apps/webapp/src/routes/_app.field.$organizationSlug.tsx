import { createFileRoute } from '@tanstack/react-router'
import { FieldDayFeature } from '#/pages/field/feature/field-day-feature'

/**
 * The field app, deliberately outside `/_app/o/$organizationSlug`.
 *
 * That layout renders the console shell: sidebar, module launcher, breadcrumb,
 * scope bar. All of it is the foreman's, and a worker on a phone would have to
 * navigate past it to reach one button. This route keeps the authentication and
 * the organization list that `/_app` provides, and nothing else.
 */
export const Route = createFileRoute('/_app/field/$organizationSlug')({
	component: FieldDayPage,
})

function FieldDayPage() {
	const { organizationSlug } = Route.useParams()

	return <FieldDayFeature organizationSlug={organizationSlug} />
}
