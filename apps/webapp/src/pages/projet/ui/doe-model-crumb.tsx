import { BreadcrumbSeparator } from '#/modules/breadcrumb-parts'

export function DoeModelCrumb({ name }: { name: string }) {
	return (
		<>
			<BreadcrumbSeparator />
			<span aria-current="page" className="truncate font-medium">
				{name}
			</span>
		</>
	)
}
