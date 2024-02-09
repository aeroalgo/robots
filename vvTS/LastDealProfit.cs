using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000CD RID: 205
	[HandlerCategory("vvTrade"), HandlerName("Профит последней закрытой позиции")]
	public class LastDealProfit : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060006F0 RID: 1776 RVA: 0x0001EF88 File Offset: 0x0001D188
		public double Execute(ISecurity src, int barNum)
		{
			IPosition lastPositionClosed = src.get_Positions().GetLastPositionClosed(barNum);
			if (lastPositionClosed != null)
			{
				return lastPositionClosed.Profit();
			}
			return 0.0;
		}
	}
}
