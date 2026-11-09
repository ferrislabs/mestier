import { Plus } from 'lucide-react'
import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import { SectionCard } from '#/components/ui/surface'
import type {
	Contract,
	ContractFileKind,
} from '#/pages/cursus/triads/feature/use-triads'
import { ContractCard } from '#/pages/cursus/triads/ui/contract-card'

interface ContractsSectionProps {
	contracts: Contract[]
	canManage: boolean
	canAdd: boolean
	isUploading: boolean
	fileError: string | null
	onAdd: () => void
	onEdit: (contract: Contract) => void
	onTutors: (contract: Contract) => void
	onClose: (contract: Contract) => void
	onOpenFile: (contract: Contract, kind: ContractFileKind) => void
	onUploadFile: (contract: Contract, kind: ContractFileKind, file: File) => void
}

export function ContractsSection({
	contracts,
	canManage,
	canAdd,
	isUploading,
	fileError,
	onAdd,
	onEdit,
	onTutors,
	onClose,
	onOpenFile,
	onUploadFile,
}: ContractsSectionProps) {
	return (
		<SectionCard>
			<div className="flex items-center justify-between gap-3 border-b px-5 py-4">
				<h2 className="font-semibold">Contrats ({contracts.length})</h2>
				{canManage && canAdd ? (
					<Button type="button" size="sm" onClick={onAdd}>
						<Plus />
						Ajouter un contrat
					</Button>
				) : null}
			</div>
			<div className="grid gap-3 p-5">
				{fileError ? (
					<p role="alert" className="text-[13px] text-state-alert">
						{fileError}
					</p>
				) : null}
				{contracts.length === 0 ? (
					<EmptyState title="Aucun contrat" />
				) : (
					contracts.map((contract) => (
						<ContractCard
							key={contract.id}
							contract={contract}
							canManage={canManage}
							isUploading={isUploading}
							onEdit={() => onEdit(contract)}
							onTutors={() => onTutors(contract)}
							onClose={() => onClose(contract)}
							onOpenFile={(kind) => onOpenFile(contract, kind)}
							onUploadFile={(kind, file) => onUploadFile(contract, kind, file)}
						/>
					))
				)}
			</div>
		</SectionCard>
	)
}
