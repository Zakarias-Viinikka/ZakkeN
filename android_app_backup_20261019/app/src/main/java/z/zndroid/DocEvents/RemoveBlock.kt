package z.zndroid.DocEvents

typealias RemoveBlockCtx = DeleteBlockCtx

object RemoveBlock {
    suspend fun execute(ctx: RemoveBlockCtx): Result<Unit> {
        return DeleteBlock.execute(ctx)
    }
}
