using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000072 RID: 114
	[HandlerCategory("vvIndicators")]
	public class ZigZag : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600040A RID: 1034 RVA: 0x00015850 File Offset: 0x00013A50
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> data = this.Context.GetData("hhv", new string[]
			{
				this.ExtDepth.ToString(),
				sec.get_CacheName()
			}, () => Series.Highest(sec.get_HighPrices(), this.ExtDepth));
			IList<double> data2 = this.Context.GetData("llv", new string[]
			{
				this.ExtDepth.ToString(),
				sec.get_CacheName()
			}, () => Series.Lowest(sec.get_LowPrices(), this.ExtDepth));
			IList<double> lowPrices = sec.get_LowPrices();
			IList<double> highPrices = sec.get_HighPrices();
			IList<double> list = new List<double>(lowPrices.Count);
			IList<double> list2 = new List<double>(lowPrices.Count);
			new List<double>(lowPrices.Count);
			double num = 0.0;
			double num2 = 0.0;
			for (int i = 0; i < lowPrices.Count; i++)
			{
				double item;
				double item2;
				if (i < this.ExtDepth)
				{
					item = 0.0;
					item2 = 0.0;
				}
				else
				{
					double num3 = data2[i];
					if (num3 == num2)
					{
						num3 = 0.0;
					}
					else
					{
						num2 = num3;
						if (lowPrices[i] - num3 > (double)this.ExtDeviation)
						{
							num3 = 0.0;
						}
						else
						{
							for (int j = 1; j <= this.ExtBackstep; j++)
							{
								double num4 = list2[i - j];
								if (num4 != 0.0 && num4 > num3)
								{
									list2[i - j] = 0.0;
								}
							}
						}
					}
					item = num3;
					double num5 = data[i];
					if (num5 == num)
					{
						num5 = 0.0;
					}
					else
					{
						num = num5;
						if (num5 - highPrices[i] > (double)this.ExtDeviation)
						{
							num5 = 0.0;
						}
						else
						{
							for (int j = 1; j <= this.ExtBackstep; j++)
							{
								double num6 = list[i - j];
								if (num6 != 0.0 && num6 < num5)
								{
									list[i - j] = 0.0;
								}
							}
						}
					}
					item2 = num5;
				}
				list.Add(item2);
				list2.Add(item);
			}
			num = -1.0;
			int index = -1;
			num2 = -1.0;
			int index2 = -1;
			for (int k = this.ExtDepth; k < lowPrices.Count; k++)
			{
				double num7 = list2[k];
				double num8 = list[k];
				if (num7 != 0.0 || num8 != 0.0)
				{
					if (num8 != 0.0)
					{
						if (num > 0.0)
						{
							if (num < num8)
							{
								list[index] = 0.0;
							}
							else
							{
								list[k] = 0.0;
							}
						}
						if (num < num8 || num < 0.0)
						{
							num = num8;
							index = k;
						}
						num2 = -1.0;
					}
					if (num7 != 0.0)
					{
						if (num2 > 0.0)
						{
							if (num2 > num7)
							{
								list2[index2] = 0.0;
							}
							else
							{
								list2[k] = 0.0;
							}
						}
						if (num7 < num2 || num2 < 0.0)
						{
							num2 = num7;
							index2 = k;
						}
						num = -1.0;
					}
				}
			}
			for (int l = 0; l < lowPrices.Count; l++)
			{
				if (l < this.ExtDepth)
				{
					list2[l] = 0.0;
				}
				else
				{
					double num4 = list[l];
					if (num4 != 0.0)
					{
						list2[l] = num4;
					}
				}
			}
			return list2;
		}

		// Token: 0x1700015E RID: 350
		public IContext Context
		{
			// Token: 0x0600040B RID: 1035 RVA: 0x00015C73 File Offset: 0x00013E73
			get;
			// Token: 0x0600040C RID: 1036 RVA: 0x00015C7B File Offset: 0x00013E7B
			set;
		}

		// Token: 0x1700015D RID: 349
		[HandlerParameter(true, "3", Min = "1", Max = "10", Step = "3")]
		public int ExtBackstep
		{
			// Token: 0x06000408 RID: 1032 RVA: 0x000157FB File Offset: 0x000139FB
			get;
			// Token: 0x06000409 RID: 1033 RVA: 0x00015803 File Offset: 0x00013A03
			set;
		}

		// Token: 0x1700015B RID: 347
		[HandlerParameter(true, "12", Min = "1", Max = "100", Step = "1")]
		public int ExtDepth
		{
			// Token: 0x06000404 RID: 1028 RVA: 0x000157D9 File Offset: 0x000139D9
			get;
			// Token: 0x06000405 RID: 1029 RVA: 0x000157E1 File Offset: 0x000139E1
			set;
		}

		// Token: 0x1700015C RID: 348
		[HandlerParameter(true, "5", Min = "1", Max = "10", Step = "1")]
		public int ExtDeviation
		{
			// Token: 0x06000406 RID: 1030 RVA: 0x000157EA File Offset: 0x000139EA
			get;
			// Token: 0x06000407 RID: 1031 RVA: 0x000157F2 File Offset: 0x000139F2
			set;
		}
	}
}
