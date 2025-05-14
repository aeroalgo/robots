using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000A6 RID: 166
	[HandlerCategory("vvPosClose"), HandlerName("SuperATRtrail")]
	public class SuperTrailStopOnATR : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600060C RID: 1548 RVA: 0x0001CD04 File Offset: 0x0001AF04
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			ISecurity security = pos.get_Security();
			double num = pos.OpenMFEPct(barNum);
			double num6;
			if (num > this.TrailEnable)
			{
				IList<double> highPrices = security.get_HighPrices();
				IList<double> lowPrices = security.get_LowPrices();
				IList<double> closePrices = security.get_ClosePrices();
				IList<double> openPrices = security.get_OpenPrices();
				double num2 = TrailStop.CalcATR(security, this.ATRperiod, barNum, 0);
				double num3 = lowPrices[pos.FindLowBar(barNum)];
				double num4 = highPrices[pos.FindHighBar(barNum)];
				double num5 = num2 * this.ATRcoef;
				if (this.FastModeATRcoef > 0.0 && this.CoefBigBarToATR > 0.0 && highPrices[barNum] - lowPrices[barNum] > num2 * this.CoefBigBarToATR)
				{
					if (pos.get_IsLong() && closePrices[barNum] > openPrices[barNum])
					{
						num5 = num2 * this.FastModeATRcoef;
					}
					if (!pos.get_IsLong() && closePrices[barNum] < openPrices[barNum])
					{
						num5 = num2 * this.FastModeATRcoef;
					}
				}
				switch (this.PriceType)
				{
				case 1:
					num6 = (pos.get_IsLong() ? (closePrices[barNum] - num5) : (closePrices[barNum] + num5));
					break;
				case 2:
					num6 = (pos.get_IsLong() ? (num4 - num5) : (num4 + num5));
					break;
				case 3:
					num6 = (pos.get_IsLong() ? (num3 - num5) : (num3 + num5));
					break;
				case 4:
					num6 = (pos.get_IsLong() ? ((num4 + num3) / 2.0 - num5) : ((num4 + num3) / 2.0 + num5));
					break;
				default:
					num6 = (pos.get_IsLong() ? (num4 - num5) : (num3 + num5));
					break;
				}
			}
			else
			{
				double num5 = -(this.StopLoss / 100.0);
				num6 = pos.get_EntryPrice() * (1.0 + (pos.get_IsLong() ? num5 : (-num5)));
			}
			double stop = pos.GetStop(barNum);
			if (stop != 0.0)
			{
				num6 = (pos.get_IsLong() ? Math.Max(num6, stop) : Math.Min(num6, stop));
			}
			if (this.StopStep > 0.0 && this.BarsForStopStep > 0 && TrailStop.IsStopStoned(pos, this.BarsForStopStep, barNum))
			{
				num6 *= 1.0 + (pos.get_IsLong() ? (this.StopStep / 100.0) : (-(this.StopStep / 100.0)));
			}
			return num6;
		}

		// Token: 0x1700020E RID: 526
		[HandlerParameter(true, "2.0", Min = "1.0", Max = "5.0", Step = "0.1", Name = "Коэф. ATR")]
		public double ATRcoef
		{
			// Token: 0x060005FE RID: 1534 RVA: 0x0001CC8D File Offset: 0x0001AE8D
			get;
			// Token: 0x060005FF RID: 1535 RVA: 0x0001CC95 File Offset: 0x0001AE95
			set;
		}

		// Token: 0x1700020F RID: 527
		[HandlerParameter(true, "5", Min = "1", Max = "10", Step = "1", Name = "Период расч. ATR")]
		public int ATRperiod
		{
			// Token: 0x06000600 RID: 1536 RVA: 0x0001CC9E File Offset: 0x0001AE9E
			get;
			// Token: 0x06000601 RID: 1537 RVA: 0x0001CCA6 File Offset: 0x0001AEA6
			set;
		}

		// Token: 0x17000213 RID: 531
		[HandlerParameter(true, "0", Min = "0", Max = "10", Step = "1", Name = "Перенос стопа после(б.)")]
		public int BarsForStopStep
		{
			// Token: 0x06000608 RID: 1544 RVA: 0x0001CCE2 File Offset: 0x0001AEE2
			get;
			// Token: 0x06000609 RID: 1545 RVA: 0x0001CCEA File Offset: 0x0001AEEA
			set;
		}

		// Token: 0x17000211 RID: 529
		[HandlerParameter(true, "0", Min = "1.5", Max = "4", Step = "0.1", Name = "Коэф. больш.бар/ATR")]
		public double CoefBigBarToATR
		{
			// Token: 0x06000604 RID: 1540 RVA: 0x0001CCC0 File Offset: 0x0001AEC0
			get;
			// Token: 0x06000605 RID: 1541 RVA: 0x0001CCC8 File Offset: 0x0001AEC8
			set;
		}

		// Token: 0x17000210 RID: 528
		[HandlerParameter(true, "0", Min = "0", Max = "10", Step = "1", Name = "Коэф.ATR для больш.бара")]
		public double FastModeATRcoef
		{
			// Token: 0x06000602 RID: 1538 RVA: 0x0001CCAF File Offset: 0x0001AEAF
			get;
			// Token: 0x06000603 RID: 1539 RVA: 0x0001CCB7 File Offset: 0x0001AEB7
			set;
		}

		// Token: 0x17000214 RID: 532
		[HandlerParameter(true, "0", Min = "0", Max = "4", Step = "1", Name = "Тип цены")]
		public int PriceType
		{
			// Token: 0x0600060A RID: 1546 RVA: 0x0001CCF3 File Offset: 0x0001AEF3
			get;
			// Token: 0x0600060B RID: 1547 RVA: 0x0001CCFB File Offset: 0x0001AEFB
			set;
		}

		// Token: 0x1700020C RID: 524
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "0.7", Step = "0.1", Name = "Stop Loss")]
		public double StopLoss
		{
			// Token: 0x060005FA RID: 1530 RVA: 0x0001CC6B File Offset: 0x0001AE6B
			get;
			// Token: 0x060005FB RID: 1531 RVA: 0x0001CC73 File Offset: 0x0001AE73
			set;
		}

		// Token: 0x17000212 RID: 530
		[HandlerParameter(true, "0", Min = "0", Max = "0.5", Step = "0.05", Name = "Шаг переноса стопа")]
		public double StopStep
		{
			// Token: 0x06000606 RID: 1542 RVA: 0x0001CCD1 File Offset: 0x0001AED1
			get;
			// Token: 0x06000607 RID: 1543 RVA: 0x0001CCD9 File Offset: 0x0001AED9
			set;
		}

		// Token: 0x1700020D RID: 525
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "0.7", Step = "0.1", Name = "Trail Enable")]
		public double TrailEnable
		{
			// Token: 0x060005FC RID: 1532 RVA: 0x0001CC7C File Offset: 0x0001AE7C
			get;
			// Token: 0x060005FD RID: 1533 RVA: 0x0001CC84 File Offset: 0x0001AE84
			set;
		}
	}
}
