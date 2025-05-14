using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000096 RID: 150
	[HandlerCategory("vvPosClose"), HandlerName("TrailStopStd Pct")]
	public class TrailStop : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x06000569 RID: 1385 RVA: 0x0001B47C File Offset: 0x0001967C
		public static double CalcATR(ISecurity sec, int atrperiod, int barnum, int AtrCalcSpeed = 0)
		{
			if (AtrCalcSpeed < 0 || AtrCalcSpeed > 1)
			{
				AtrCalcSpeed = 0;
			}
			IList<double> highPrices = sec.get_HighPrices();
			IList<double> lowPrices = sec.get_LowPrices();
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> arg_26_0 = sec.get_OpenPrices();
			double num = 0.0;
			double num2 = 0.0;
			int num3 = barnum - atrperiod + 1;
			if (num3 < 1)
			{
				num3 = 1;
			}
			if (AtrCalcSpeed > atrperiod || AtrCalcSpeed < 0)
			{
				AtrCalcSpeed = 0;
			}
			for (int i = 0; i < atrperiod; i = i + 1 + ((AtrCalcSpeed == 0) ? 0 : (AtrCalcSpeed - 1)))
			{
				double num4 = Math.Max(Math.Max(highPrices[num3 + i] - lowPrices[num3 + i], highPrices[num3 + i] - closePrices[num3 + i - 1]), lowPrices[num3 + i] - closePrices[num3 + i - 1]);
				num += num4;
				num2 += 1.0;
			}
			return num / num2;
		}

		// Token: 0x0600056A RID: 1386 RVA: 0x0001B574 File Offset: 0x00019774
		public static double CalcATRnoGaps(ISecurity sec, int atrperiod, int barnum, bool nogaps = false)
		{
			IList<double> highPrices = sec.get_HighPrices();
			IList<double> lowPrices = sec.get_LowPrices();
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> arg_1B_0 = sec.get_OpenPrices();
			double num = 0.0;
			double num2 = 0.0;
			int num3 = barnum - atrperiod + 1;
			if (num3 < 1)
			{
				num3 = 1;
			}
			for (int i = 0; i < atrperiod; i++)
			{
				double num4 = Math.Max(Math.Max(highPrices[num3 + i] - lowPrices[num3 + i], highPrices[num3 + i] - closePrices[num3 + i - 1]), lowPrices[num3 + i] - closePrices[num3 + i - 1]);
				num += num4;
				num2 += 1.0;
			}
			return num / num2;
		}

		// Token: 0x06000567 RID: 1383 RVA: 0x0001B2E0 File Offset: 0x000194E0
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
			if (num >= this.TrailEnable)
			{
				num2 = (num - this.TrailLoss) / 100.0;
				num3 = pos.get_EntryPrice() * (1.0 + (pos.get_IsLong() ? num2 : (-num2)));
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

		// Token: 0x06000568 RID: 1384 RVA: 0x0001B41C File Offset: 0x0001961C
		public static bool IsStopStoned(IPosition _pos, int _BarsCheck, int _barNum)
		{
			double num = _pos.GetStop(_barNum);
			if (num == 0.0)
			{
				return false;
			}
			if (_BarsCheck < 1)
			{
				return false;
			}
			if (_barNum < _BarsCheck)
			{
				return false;
			}
			if (_BarsCheck == 1)
			{
				return true;
			}
			double stop = _pos.GetStop(_barNum - 1);
			int i = 1;
			while (i < _BarsCheck)
			{
				if (num != stop)
				{
					return false;
				}
				i++;
				num = stop;
				stop = _pos.GetStop(_barNum - i);
			}
			return true;
		}

		// Token: 0x170001D2 RID: 466
		[HandlerParameter(true, "0", Min = "0", Max = "25", Step = "1", Name = "Перенос стопа после(б.)")]
		public int BarsForStopStep
		{
			// Token: 0x06000563 RID: 1379 RVA: 0x0001B2BD File Offset: 0x000194BD
			get;
			// Token: 0x06000564 RID: 1380 RVA: 0x0001B2C5 File Offset: 0x000194C5
			set;
		}

		// Token: 0x170001CF RID: 463
		[HandlerParameter(true, "1.7", Min = "0.05", Max = "2", Step = "0.05", Name = "Stop Loss")]
		public double StopLoss
		{
			// Token: 0x0600055D RID: 1373 RVA: 0x0001B28A File Offset: 0x0001948A
			get;
			// Token: 0x0600055E RID: 1374 RVA: 0x0001B292 File Offset: 0x00019492
			set;
		}

		// Token: 0x170001D3 RID: 467
		[HandlerParameter(true, "0", Min = "0", Max = "0.5", Step = "0.05", Name = "Шаг переноса стопа")]
		public double StopStep
		{
			// Token: 0x06000565 RID: 1381 RVA: 0x0001B2CE File Offset: 0x000194CE
			get;
			// Token: 0x06000566 RID: 1382 RVA: 0x0001B2D6 File Offset: 0x000194D6
			set;
		}

		// Token: 0x170001D0 RID: 464
		[HandlerParameter(true, "0.3", Min = "0.05", Max = "1", Step = "0.05", Name = "Trail Enable")]
		public double TrailEnable
		{
			// Token: 0x0600055F RID: 1375 RVA: 0x0001B29B File Offset: 0x0001949B
			get;
			// Token: 0x06000560 RID: 1376 RVA: 0x0001B2A3 File Offset: 0x000194A3
			set;
		}

		// Token: 0x170001D1 RID: 465
		[HandlerParameter(true, "0.6", Min = "0.05", Max = "1", Step = "0.05", Name = "Trail Loss")]
		public double TrailLoss
		{
			// Token: 0x06000561 RID: 1377 RVA: 0x0001B2AC File Offset: 0x000194AC
			get;
			// Token: 0x06000562 RID: 1378 RVA: 0x0001B2B4 File Offset: 0x000194B4
			set;
		}
	}
}
