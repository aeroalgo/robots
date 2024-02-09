using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000B5 RID: 181
	[HandlerCategory("vvPosClose"), HandlerName("OnBarsStopLoss")]
	public class OnBarsStopLoss : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x06000687 RID: 1671 RVA: 0x0001DC10 File Offset: 0x0001BE10
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
			double result = pos.get_EntryPrice() + (pos.get_IsLong() ? (-this.StopLoss) : this.StopLoss);
			if (num > this.StopLoss2Bars)
			{
				result = pos.get_EntryPrice() + (pos.get_IsLong() ? (-this.StopLoss2) : this.StopLoss2);
			}
			return result;
		}

		// Token: 0x17000240 RID: 576
		[HandlerParameter(true, "200", Min = "10", Max = "200", Step = "10", Name = "СтопЛосс1")]
		public double StopLoss
		{
			// Token: 0x06000681 RID: 1665 RVA: 0x0001DBDC File Offset: 0x0001BDDC
			get;
			// Token: 0x06000682 RID: 1666 RVA: 0x0001DBE4 File Offset: 0x0001BDE4
			set;
		}

		// Token: 0x17000241 RID: 577
		[HandlerParameter(true, "400", Min = "10", Max = "200", Step = "10", Name = "СтопЛосс2")]
		public double StopLoss2
		{
			// Token: 0x06000683 RID: 1667 RVA: 0x0001DBED File Offset: 0x0001BDED
			get;
			// Token: 0x06000684 RID: 1668 RVA: 0x0001DBF5 File Offset: 0x0001BDF5
			set;
		}

		// Token: 0x17000242 RID: 578
		[HandlerParameter(true, "5", Min = "2", Max = "10", Step = "1", Name = "СЛ2 баров")]
		public int StopLoss2Bars
		{
			// Token: 0x06000685 RID: 1669 RVA: 0x0001DBFE File Offset: 0x0001BDFE
			get;
			// Token: 0x06000686 RID: 1670 RVA: 0x0001DC06 File Offset: 0x0001BE06
			set;
		}
	}
}
