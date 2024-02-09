using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000F6 RID: 246
	[HandlerCategory("vvTrade"), HandlerName("Накопленный убыток")]
	public class CumulatedLoss : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000752 RID: 1874 RVA: 0x00020848 File Offset: 0x0001EA48
		public double Execute(ISecurity src, int barNum)
		{
			if (barNum == 0)
			{
				return 0.0;
			}
			IPosition lastPositionClosed = src.get_Positions().GetLastPositionClosed(barNum);
			if (lastPositionClosed == null)
			{
				return 0.0;
			}
			int arg_2D_0 = lastPositionClosed.get_ExitBarNum();
			lastPositionClosed.Profit();
			return (double)lastPositionClosed.get_EntryBarNum();
		}
	}
}
