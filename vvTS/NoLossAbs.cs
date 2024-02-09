using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000AC RID: 172
	[HandlerCategory("vvPosClose"), HandlerName("Безубыток")]
	public class NoLossAbs : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x06000651 RID: 1617 RVA: 0x0001D70C File Offset: 0x0001B90C
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			double num = pos.OpenMFE(barNum);
			double result = (double)(pos.get_IsLong() ? 0 : 1000000);
			if (num > this.Profit)
			{
				result = pos.get_EntryPrice() + (pos.get_IsLong() ? this.NoLoss : (-this.NoLoss));
			}
			return result;
		}

		// Token: 0x17000230 RID: 560
		[HandlerParameter(true, "100", Min = "10", Max = "200", Step = "10", Name = "Безубыток")]
		public double NoLoss
		{
			// Token: 0x0600064F RID: 1615 RVA: 0x0001D6FB File Offset: 0x0001B8FB
			get;
			// Token: 0x06000650 RID: 1616 RVA: 0x0001D703 File Offset: 0x0001B903
			set;
		}

		// Token: 0x1700022F RID: 559
		[HandlerParameter(true, "400", Min = "10", Max = "500", Step = "10")]
		public double Profit
		{
			// Token: 0x0600064D RID: 1613 RVA: 0x0001D6EA File Offset: 0x0001B8EA
			get;
			// Token: 0x0600064E RID: 1614 RVA: 0x0001D6F2 File Offset: 0x0001B8F2
			set;
		}
	}
}
