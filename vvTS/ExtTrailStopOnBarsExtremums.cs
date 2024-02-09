using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200009C RID: 156
	[HandlerCategory("vvPosClose"), HandlerName("Трейл по теням Ext Pct")]
	public class ExtTrailStopOnBarsExtremums : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x060005A8 RID: 1448 RVA: 0x0001BE64 File Offset: 0x0001A064
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			double num = pos.OpenMFEPct(barNum);
			double num2 = -(this.StopLoss / 100.0);
			double num3 = pos.get_EntryPrice() * (1.0 + (pos.get_IsLong() ? num2 : (-num2)));
			double val = num3;
			if (num > this.TrailEnable)
			{
				ISecurity security = pos.get_Security();
				IList<double> highPrices = security.get_HighPrices();
				IList<double> lowPrices = security.get_LowPrices();
				IList<Bar> arg_78_0 = security.get_Bars();
				int num4 = this.BarsCount;
				if (num4 > barNum)
				{
					num4 = barNum;
				}
				double num5 = highPrices[barNum - num4];
				double num6 = lowPrices[barNum - num4];
				for (int i = barNum - num4; i <= barNum; i++)
				{
					if (highPrices[i] > num5)
					{
						num5 = highPrices[i];
					}
					if (lowPrices[i] < num6)
					{
						num6 = lowPrices[i];
					}
				}
				num3 = (pos.get_IsLong() ? num6 : num5);
			}
			num3 = (pos.get_IsLong() ? Math.Max(num3, val) : Math.Min(num3, val));
			double stop = pos.GetStop(barNum);
			if (stop != 0.0)
			{
				num3 = (pos.get_IsLong() ? Math.Max(num3, stop) : Math.Min(num3, stop));
			}
			if (this.StopStep > 0.0 && this.BarsForStopStep > 0 && TrailStop.IsStopStoned(pos, this.BarsForStopStep, barNum))
			{
				num3 *= 1.0 + (pos.get_IsLong() ? (this.StopStep / 100.0) : (-(this.StopStep / 100.0)));
			}
			return num3;
		}

		// Token: 0x170001EA RID: 490
		[HandlerParameter(true, "4", Min = "1", Max = "25", Step = "1", Name = "Bars Count")]
		public int BarsCount
		{
			// Token: 0x060005A2 RID: 1442 RVA: 0x0001BE30 File Offset: 0x0001A030
			get;
			// Token: 0x060005A3 RID: 1443 RVA: 0x0001BE38 File Offset: 0x0001A038
			set;
		}

		// Token: 0x170001EB RID: 491
		[HandlerParameter(true, "0", Min = "0", Max = "10", Step = "1", Name = "Перенос стопа после(б.)")]
		public int BarsForStopStep
		{
			// Token: 0x060005A4 RID: 1444 RVA: 0x0001BE41 File Offset: 0x0001A041
			get;
			// Token: 0x060005A5 RID: 1445 RVA: 0x0001BE49 File Offset: 0x0001A049
			set;
		}

		// Token: 0x170001E8 RID: 488
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "0.6", Step = "0.1", Name = "Stop Loss")]
		public double StopLoss
		{
			// Token: 0x0600059E RID: 1438 RVA: 0x0001BE0E File Offset: 0x0001A00E
			get;
			// Token: 0x0600059F RID: 1439 RVA: 0x0001BE16 File Offset: 0x0001A016
			set;
		}

		// Token: 0x170001EC RID: 492
		[HandlerParameter(true, "0", Min = "0", Max = "0.5", Step = "0.05", Name = "Шаг переноса стопа")]
		public double StopStep
		{
			// Token: 0x060005A6 RID: 1446 RVA: 0x0001BE52 File Offset: 0x0001A052
			get;
			// Token: 0x060005A7 RID: 1447 RVA: 0x0001BE5A File Offset: 0x0001A05A
			set;
		}

		// Token: 0x170001E9 RID: 489
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "0.6", Step = "0.1", Name = "Trail Enable")]
		public double TrailEnable
		{
			// Token: 0x060005A0 RID: 1440 RVA: 0x0001BE1F File Offset: 0x0001A01F
			get;
			// Token: 0x060005A1 RID: 1441 RVA: 0x0001BE27 File Offset: 0x0001A027
			set;
		}
	}
}
