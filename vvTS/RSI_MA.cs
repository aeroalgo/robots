using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000130 RID: 304
	[HandlerCategory("vvRSI"), HandlerName("RSIonEMA")]
	public class RSI_MA : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060008F8 RID: 2296 RVA: 0x00025F00 File Offset: 0x00024100
		public IList<double> Execute(ISecurity sec)
		{
			return RSI_MA.GenRSI_MA(sec, this.RSIperiod, this.Filter, this.Context);
		}

		// Token: 0x060008F7 RID: 2295 RVA: 0x00025BD4 File Offset: 0x00023DD4
		public static IList<double> GenRSI_MA(ISecurity src, int _rsiperiod, int _filter, IContext context)
		{
			int count = src.get_Bars().Count;
			if (count < _rsiperiod)
			{
				return null;
			}
			IList<double> closePrices = src.get_ClosePrices();
			IList<double> highPrices = src.get_HighPrices();
			IList<double> lowPrices = src.get_LowPrices();
			double tick = src.get_Tick();
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] weighprice = new double[count];
			double[] array5 = new double[count];
			for (int i = 0; i < count; i++)
			{
				weighprice[i] = (closePrices[i] * 2.0 + highPrices[i] + lowPrices[i]) / 4.0;
			}
			IList<double> data = context.GetData("ema", new string[]
			{
				_rsiperiod.ToString(),
				weighprice.GetHashCode().ToString()
			}, () => Series.EMA(weighprice, _rsiperiod));
			for (int j = 1; j < count; j++)
			{
				array5[j] = data[j] - data[j - 1];
			}
			for (int k = 1; k < count; k++)
			{
				double num = 0.0;
				double num2 = 0.0;
				double num4;
				double num5;
				if (k == _rsiperiod - 1)
				{
					for (int l = 1; l <= k; l++)
					{
						double num3 = closePrices[l] - closePrices[l - 1];
						if (num3 > 0.0)
						{
							num2 += num3;
						}
						else
						{
							num -= num3;
						}
					}
					num4 = num2 / (double)_rsiperiod;
					num5 = num / (double)_rsiperiod;
				}
				else
				{
					double num3 = closePrices[k] - closePrices[k - 1];
					if (num3 > 0.0)
					{
						num2 = num3;
					}
					else
					{
						num = -num3;
					}
					num4 = (array[k - 1] * (double)(_rsiperiod - 1) + num2) / (double)_rsiperiod;
					num5 = (array2[k - 1] * (double)(_rsiperiod - 1) + num) / (double)_rsiperiod;
				}
				array[k] = num4;
				array2[k] = num5;
				if (num5 == 0.0)
				{
					array3[k] = 0.0;
				}
				else
				{
					array3[k] = 100.0 - 100.0 / (1.0 + num4 / num5);
				}
				array4[k] = array3[k] * array5[k] / tick;
			}
			IList<double> list = JMA.GenJMA(array4, _filter, 100);
			double[] array6 = new double[count];
			for (int m = 0; m < count; m++)
			{
				if (list[m] > 100.0)
				{
					array6[m] = 100.0;
				}
				else if (list[m] < 0.0)
				{
					array6[m] = 0.0;
				}
				else
				{
					array6[m] = list[m];
				}
			}
			return array6;
		}

		// Token: 0x170002E2 RID: 738
		public IContext Context
		{
			// Token: 0x060008F9 RID: 2297 RVA: 0x00025F1A File Offset: 0x0002411A
			get;
			// Token: 0x060008FA RID: 2298 RVA: 0x00025F22 File Offset: 0x00024122
			set;
		}

		// Token: 0x170002E1 RID: 737
		[HandlerParameter(true, "3", Min = "1", Max = "30", Step = "1")]
		public int Filter
		{
			// Token: 0x060008F5 RID: 2293 RVA: 0x00025BA7 File Offset: 0x00023DA7
			get;
			// Token: 0x060008F6 RID: 2294 RVA: 0x00025BAF File Offset: 0x00023DAF
			set;
		}

		// Token: 0x170002E0 RID: 736
		[HandlerParameter(true, "14", Min = "6", Max = "30", Step = "1")]
		public int RSIperiod
		{
			// Token: 0x060008F3 RID: 2291 RVA: 0x00025B96 File Offset: 0x00023D96
			get;
			// Token: 0x060008F4 RID: 2292 RVA: 0x00025B9E File Offset: 0x00023D9E
			set;
		}
	}
}
