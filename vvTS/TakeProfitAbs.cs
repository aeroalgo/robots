using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000B3 RID: 179
	[HandlerCategory("vvPosClose"), HandlerName("TakeProfit Abs")]
	public class TakeProfitAbs : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600067B RID: 1659 RVA: 0x0001DB18 File Offset: 0x0001BD18
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			pos.OpenMFE(barNum);
			return pos.get_EntryPrice() + (pos.get_IsLong() ? this.TakeProfit : (-this.TakeProfit));
		}

		// Token: 0x1700023E RID: 574
		[HandlerParameter(true, "500", Min = "10", Max = "500", Step = "10")]
		public double TakeProfit
		{
			// Token: 0x06000679 RID: 1657 RVA: 0x0001DB06 File Offset: 0x0001BD06
			get;
			// Token: 0x0600067A RID: 1658 RVA: 0x0001DB0E File Offset: 0x0001BD0E
			set;
		}
	}
}
