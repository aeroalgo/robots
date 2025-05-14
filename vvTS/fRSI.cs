using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200012E RID: 302
	[HandlerCategory("vvRSI"), HandlerName("Figurelli RSI")]
	public class fRSI : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060008E2 RID: 2274 RVA: 0x00025957 File Offset: 0x00023B57
		public IList<double> Execute(IList<double> src)
		{
			return fRSI.GenFRSI(src, this.Period, this.Gain, this.Context, this.preSmooth, this.postSmooth);
		}

		// Token: 0x060008E1 RID: 2273 RVA: 0x00025694 File Offset: 0x00023894
		public static IList<double> GenFRSI(IList<double> src, int _Period, double _Gain, IContext ctx, int _presmooth, int _postsmooth)
		{
			int count = src.Count;
			IList<double> list = src;
			if (_Gain < 1.0)
			{
				_Gain = (double)(_Period / 5);
			}
			if (_presmooth > 0)
			{
				list = ctx.GetData("sma", new string[]
				{
					_presmooth.ToString(),
					src.GetHashCode().ToString()
				}, () => Series.SMA(src, _presmooth));
			}
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			for (int i = _Period; i < count; i++)
			{
				double num = 0.0;
				double num2 = 0.0;
				double num4;
				double num5;
				if (i == _Period)
				{
					for (int j = 2; j < i; j++)
					{
						double num3 = list[j] - list[j - 1];
						if (num3 > 0.0)
						{
							num2 += num3;
						}
						else
						{
							num -= num3;
						}
					}
					num4 = num2 / (double)_Period;
					num5 = num / (double)_Period;
				}
				else
				{
					double num3 = list[i] - list[i - 1];
					if (num3 > 0.0)
					{
						num2 = num3;
					}
					else
					{
						num = -num3;
					}
					num4 = (array2[i - 1] * (double)(_Period - 1) + num2) / (double)_Period;
					num5 = (array3[i - 1] * (double)(_Period - 1) + num) / (double)_Period;
				}
				array2[i] = num4;
				array3[i] = num5;
				if (num5 == 0.0)
				{
					array[i] = 0.0;
				}
				else
				{
					array[i] = 100.0 - 100.0 / (1.0 + num4 / num5);
				}
				array[i] -= 50.0;
				array[i] *= _Gain;
				if (array[i] < -50.0)
				{
					array[i] = -50.0;
					_Gain -= 0.1;
					if (_Gain < 1.0)
					{
						_Gain = 1.0;
					}
				}
				if (array[i] > 50.0)
				{
					array[i] = 50.0;
					_Gain -= 0.1;
					if (_Gain < 1.0)
					{
						_Gain = 1.0;
					}
				}
				array[i] += 50.0;
			}
			IList<double> result = array;
			if (_postsmooth > 0)
			{
				result = SMA.GenSMA(array, _postsmooth);
			}
			return result;
		}

		// Token: 0x170002DA RID: 730
		public IContext Context
		{
			// Token: 0x060008E3 RID: 2275 RVA: 0x0002597D File Offset: 0x00023B7D
			get;
			// Token: 0x060008E4 RID: 2276 RVA: 0x00025985 File Offset: 0x00023B85
			set;
		}

		// Token: 0x170002D7 RID: 727
		[HandlerParameter(true, "0", Min = "0", Max = "10", Step = "1")]
		public double Gain
		{
			// Token: 0x060008DB RID: 2267 RVA: 0x00025643 File Offset: 0x00023843
			get;
			// Token: 0x060008DC RID: 2268 RVA: 0x0002564B File Offset: 0x0002384B
			set;
		}

		// Token: 0x170002D6 RID: 726
		[HandlerParameter(true, "120", Min = "10", Max = "300", Step = "10")]
		public int Period
		{
			// Token: 0x060008D9 RID: 2265 RVA: 0x00025632 File Offset: 0x00023832
			get;
			// Token: 0x060008DA RID: 2266 RVA: 0x0002563A File Offset: 0x0002383A
			set;
		}

		// Token: 0x170002D9 RID: 729
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "10")]
		public int postSmooth
		{
			// Token: 0x060008DF RID: 2271 RVA: 0x00025665 File Offset: 0x00023865
			get;
			// Token: 0x060008E0 RID: 2272 RVA: 0x0002566D File Offset: 0x0002386D
			set;
		}

		// Token: 0x170002D8 RID: 728
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "10")]
		public int preSmooth
		{
			// Token: 0x060008DD RID: 2269 RVA: 0x00025654 File Offset: 0x00023854
			get;
			// Token: 0x060008DE RID: 2270 RVA: 0x0002565C File Offset: 0x0002385C
			set;
		}
	}
}
