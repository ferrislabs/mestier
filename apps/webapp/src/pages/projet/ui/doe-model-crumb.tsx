import { BreadcrumbItem, BreadcrumbSeparator } from '#/components/ui/breadcrumb'

export function DoeModelCrumb({ name }: { name: string }) {
	return (
		<>
			<BreadcrumbSeparator />
			<BreadcrumbItem className="min-w-0">
				<span aria-current="page" className="truncate font-medium">
					{name}
				</span>
			</BreadcrumbItem>
		</>
	)
}
