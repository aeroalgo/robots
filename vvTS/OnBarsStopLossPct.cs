using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000B6 RID: 182
	[HandlerCategory("vvPosClose"), HandlerName("OnBarsStopLoss(%)")]
	public class OnBarsStopLossPct : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600068F RID: 1679 RVA: 0x0001DCD4 File Offset: 0x0001BED4
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			if (this.StopLoss < 0.0)
			{
				return 0.0;
			}
			int num = barNum - pos.get_EntryBarNum();
			double result = pos.get_EntryPrice() * (1.0 + (pos.get_IsLong() ? (-(this.StopLoss / 100.0)) : (this.StopLoss / 100.0)));
			if (num > this.StopLoss2Bars)
			{
				result = pos.get_EntryPrice() * (1.0 + (pos.get_IsLong() ? (-(this.StopLoss2 / 100.0)) : (this.StopLoss2 / 100.0)));
			}
			return result;
		}

		// Token: 0x17000243 RID: 579
		[HandlerParameter(true, "0.1", Min = "0.1", Max = "1", Step = "0.1", Name = "СтопЛосс1")]
		public double StopLoss
		{
			// Token: 0x06000689 RID: 1673 RVA: 0x0001DC9E File Offset: 0x0001BE9E
			get;
			// Token: 0x0600068A RID: 1674 RVA: 0x0001DCA6 File Offset: 0x0001BEA6
			set;
		}

		// Token: 0x17000244 RID: 580
		[HandlerParameter(true, "0.3", Min = "0.1", Max = "1", Step = "0.1", Name = "СтопЛосс2")]
		public double StopLoss2
		{
			// Token: 0x0600068B RID: 1675 RVA: 0x0001DCAF File Offset: 0x0001BEAF
			get;
			// Token: 0x0600068C RID: 1676 RVA: 0x0001DCB7 File Offset: 0x0001BEB7
			set;
		}

		// Token: 0x17000245 RID: 581
		[HandlerParameter(true, "5", Min = "2", Max = "10", Step = "1", Name = "СЛ2 баров")]
		public int StopLoss2Bars
		{
			// Token: 0x0600068D RID: 1677 RVA: 0x0001DCC0 File Offset: 0x0001BEC0
			get;
			// Token: 0x0600068E RID: 1678 RVA: 0x0001DCC8 File Offset: 0x0001BEC8
			set;
		}
	}
}
