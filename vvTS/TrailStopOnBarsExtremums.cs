using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200009A RID: 154
	[HandlerCategory("vvPosClose"), HandlerName("Трейл по теням Pct")]
	public class TrailStopOnBarsExtremums : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x06000592 RID: 1426 RVA: 0x0001BAC0 File Offset: 0x00019CC0
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			double num = pos.OpenMFEPct(barNum);
			double num2 = (0.0 - this.StopLoss) / 100.0;
			double num3 = pos.get_EntryPrice() * (1.0 + (pos.get_IsLong() ? num2 : (-num2)));
			double val = num3;
			if (num > this.TrailEnable)
			{
				ISecurity security = pos.get_Security();
				IList<double> highPrices = security.get_HighPrices();
				IList<double> lowPrices = security.get_LowPrices();
				IList<Bar> arg_81_0 = security.get_Bars();
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
			if (stop == 0.0)
			{
				return num3;
			}
			if (!pos.get_IsLong())
			{
				return Math.Min(num3, stop);
			}
			return Math.Max(num3, stop);
		}

		// Token: 0x170001E3 RID: 483
		[HandlerParameter(true, "4", Min = "1", Max = "25", Step = "1", Name = "Bars Count")]
		public int BarsCount
		{
			// Token: 0x06000590 RID: 1424 RVA: 0x0001BAAF File Offset: 0x00019CAF
			get;
			// Token: 0x06000591 RID: 1425 RVA: 0x0001BAB7 File Offset: 0x00019CB7
			set;
		}

		// Token: 0x170001E1 RID: 481
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "0.6", Step = "0.1", Name = "Stop Loss")]
		public double StopLoss
		{
			// Token: 0x0600058C RID: 1420 RVA: 0x0001BA8D File Offset: 0x00019C8D
			get;
			// Token: 0x0600058D RID: 1421 RVA: 0x0001BA95 File Offset: 0x00019C95
			set;
		}

		// Token: 0x170001E2 RID: 482
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "0.6", Step = "0.1", Name = "Trail Enable")]
		public double TrailEnable
		{
			// Token: 0x0600058E RID: 1422 RVA: 0x0001BA9E File Offset: 0x00019C9E
			get;
			// Token: 0x0600058F RID: 1423 RVA: 0x0001BAA6 File Offset: 0x00019CA6
			set;
		}
	}
}
