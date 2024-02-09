using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000CF RID: 207
	[HandlerCategory("vvTrade"), HandlerName("Лотов в последней закр. сделке")]
	public class LotsInLastTrade : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060006F6 RID: 1782 RVA: 0x0001F044 File Offset: 0x0001D244
		public double Execute(ISecurity src, int barNum)
		{
			IPosition lastPositionClosed = src.get_Positions().GetLastPositionClosed(barNum);
			if (lastPositionClosed == null)
			{
				return 0.0;
			}
			int exitBarNum = lastPositionClosed.get_ExitBarNum();
			return lastPositionClosed.Profit() / lastPositionClosed.OpenProfit(exitBarNum);
		}
	}
}
