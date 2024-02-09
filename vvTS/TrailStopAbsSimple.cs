using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000A7 RID: 167
	[HandlerCategory("vvPosClose"), HandlerName("TrailAbs простой")]
	public class TrailStopAbsSimple : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x06000612 RID: 1554 RVA: 0x0001CFBC File Offset: 0x0001B1BC
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			ISecurity security = pos.get_Security();
			double num = security.get_ClosePrices()[barNum];
			if (this.UseHighOrLow != 0)
			{
				num = ((this.UseHighOrLow > 0) ? security.get_HighPrices()[barNum] : security.get_LowPrices()[barNum]);
			}
			double num2 = num + (pos.get_IsLong() ? (-this.TrailLoss) : this.TrailLoss);
			double stop = pos.GetStop(barNum);
			if (stop == 0.0)
			{
				return num2;
			}
			if (!pos.get_IsLong())
			{
				return Math.Min(num2, stop);
			}
			return Math.Max(num2, stop);
		}

		// Token: 0x17000215 RID: 533
		[HandlerParameter(true, "50", Min = "10", Max = "600", Step = "10")]
		public double TrailLoss
		{
			// Token: 0x0600060E RID: 1550 RVA: 0x0001CF99 File Offset: 0x0001B199
			get;
			// Token: 0x0600060F RID: 1551 RVA: 0x0001CFA1 File Offset: 0x0001B1A1
			set;
		}

		// Token: 0x17000216 RID: 534
		[HandlerParameter(true, "0", Min = "-1", Max = "1", Step = "1", Name = "Close 0, High 1, Low -1")]
		public int UseHighOrLow
		{
			// Token: 0x06000610 RID: 1552 RVA: 0x0001CFAA File Offset: 0x0001B1AA
			get;
			// Token: 0x06000611 RID: 1553 RVA: 0x0001CFB2 File Offset: 0x0001B1B2
			set;
		}
	}
}
