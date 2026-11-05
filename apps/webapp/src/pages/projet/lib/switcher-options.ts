import type { Project } from '#/hooks/use-projects'
import type { SwitcherOption } from '#/pages/projet/ui/project-switcher'

export function switcherOptions(
	projects: readonly Project[],
	customers: readonly { id: string; name: string }[],
): SwitcherOption[] {
	const owners = new Map(
		customers.map((customer) => [customer.id, customer.name]),
	)
	return projects
		.map(
			(project): SwitcherOption => ({
				id: project.id,
				name: project.name,
				ownerName: project.customer_id
					? (owners.get(project.customer_id) ?? 'Client')
					: null,
				archived: Boolean(project.archived_at),
			}),
		)
		.sort(
			(a, b) =>
				Number(a.archived) - Number(b.archived) ||
				a.name.localeCompare(b.name, 'fr'),
		)
}
