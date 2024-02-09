using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200003C RID: 60
	[HandlerCategory("vvIndicators"), HandlerName("NRTR")]
	public class NRTRWATR : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600022D RID: 557 RVA: 0x0000A26C File Offset: 0x0000846C
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("NRTR", new string[]
			{
				this.Coeff.ToString(),
				this.AtrPeriod.ToString(),
				this.WATR.ToString(),
				sec.get_CacheName()
			}, () => NRTRWATR.GenNRTR(sec, this.Coeff, this.AtrPeriod, this.WATR, this.Context));
		}

		// Token: 0x0600022C RID: 556 RVA: 0x00009FBC File Offset: 0x000081BC
		public static IList<double> GenNRTR(ISecurity src, double _K, int atrperiod, bool usewatr, IContext context)
		{
			int count = src.get_Bars().Count;
			IList<double> closePrices = src.get_ClosePrices();
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			IList<double> data;
			if (usewatr)
			{
				data = context.GetData("watr", new string[]
				{
					atrperiod.ToString(),
					src.get_CacheName()
				}, () => ATR.GenWATR(src, atrperiod, 0, context));
			}
			else
			{
				data = context.GetData("atr", new string[]
				{
					atrperiod.ToString(),
					src.get_CacheName()
				}, () => ATR.GenATR(src.get_Bars(), atrperiod));
			}
			for (int i = 1; i < count; i++)
			{
				double num = data[i] * _K;
				if (array[i - 1] > 0.0)
				{
					if (closePrices[i] < array2[i - 1])
					{
						array[i] = -1.0;
						array4[i] = closePrices[i] + num;
						array3[i] = 0.0;
						array2[i] = array4[i];
					}
					else
					{
						array[i] = 1.0;
						array3[i] = closePrices[i] - num;
						array4[i] = 0.0;
						if (array3[i] > array2[i - 1])
						{
							array2[i] = array3[i];
						}
						else
						{
							array2[i] = array2[i - 1];
						}
					}
				}
				else if (closePrices[i] > array2[i - 1])
				{
					array[i] = 1.0;
					array3[i] = closePrices[i] - num;
					array4[i] = 0.0;
					array2[i] = array3[i];
				}
				else
				{
					array[i] = -1.0;
					array4[i] = closePrices[i] + num;
					array3[i] = 0.0;
					if (array4[i] < array2[i - 1])
					{
						array2[i] = array4[i];
					}
					else
					{
						array2[i] = array2[i - 1];
					}
				}
			}
			return array2;
		}

		// Token: 0x170000BA RID: 186
		[HandlerParameter(true, "10", Min = "2", Max = "20", Step = "1")]
		public int AtrPeriod
		{
			// Token: 0x06000226 RID: 550 RVA: 0x00009F4F File Offset: 0x0000814F
			get;
			// Token: 0x06000227 RID: 551 RVA: 0x00009F57 File Offset: 0x00008157
			set;
		}

		// Token: 0x170000BB RID: 187
		[HandlerParameter(true, "1", Min = "0.1", Max = "10", Step = "0.1")]
		public double Coeff
		{
			// Token: 0x06000228 RID: 552 RVA: 0x00009F60 File Offset: 0x00008160
			get;
			// Token: 0x06000229 RID: 553 RVA: 0x00009F68 File Offset: 0x00008168
			set;
		}

		// Token: 0x170000BD RID: 189
		public IContext Context
		{
			// Token: 0x0600022E RID: 558 RVA: 0x0000A2F3 File Offset: 0x000084F3
			get;
			// Token: 0x0600022F RID: 559 RVA: 0x0000A2FB File Offset: 0x000084FB
			set;
		}

		// Token: 0x170000BC RID: 188
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool WATR
		{
			// Token: 0x0600022A RID: 554 RVA: 0x00009F71 File Offset: 0x00008171
			get;
			// Token: 0x0600022B RID: 555 RVA: 0x00009F79 File Offset: 0x00008179
			set;
		}
	}
}
