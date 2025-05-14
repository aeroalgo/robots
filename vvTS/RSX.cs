using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000128 RID: 296
	[HandlerCategory("vvRSI"), HandlerName("RSX")]
	public class RSX : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000896 RID: 2198 RVA: 0x000243C4 File Offset: 0x000225C4
		public IList<double> Execute(IList<double> _data)
		{
			return this.Context.GetData("rsx", new string[]
			{
				this.length.ToString(),
				_data.GetHashCode().ToString()
			}, () => RSX.GenRSX(_data, this.length));
		}

		// Token: 0x06000895 RID: 2197 RVA: 0x00023FAC File Offset: 0x000221AC
		public static IList<double> GenRSX(IList<double> _data, int Len)
		{
			double[] array = new double[_data.Count];
			double num = 0.0;
			double num2 = 0.0;
			double num3 = 0.0;
			double num4 = 0.0;
			double num5 = 0.0;
			double num6 = 0.0;
			double num7 = 0.0;
			double num8 = 0.0;
			double num9 = 0.0;
			double num10 = 0.0;
			double num11 = 0.0;
			double num12 = 0.0;
			double num13 = 0.0;
			double num14 = 0.0;
			double num15 = 0.0;
			double num16 = 0.0;
			double num17 = 0.0;
			double num18 = 0.0;
			double num19 = 0.0;
			double num20 = 0.0;
			for (int i = 0; i < _data.Count; i++)
			{
				if (num2 == 0.0)
				{
					num2 = 1.0;
					num3 = 0.0;
					if (Len - 1 >= 5)
					{
						num = (double)Len - 1.0;
					}
					else
					{
						num = 5.0;
					}
					num6 = 100.0 * _data[i];
					num7 = 3.0 / ((double)Len + 2.0);
					num8 = 1.0 - num7;
				}
				else
				{
					if (num <= num2)
					{
						num2 = num + 1.0;
					}
					else
					{
						num2 += 1.0;
					}
					double num21 = num6;
					num6 = 100.0 * _data[i];
					double num22 = num6 - num21;
					num9 = num8 * num9 + num7 * num22;
					num10 = num7 * num9 + num8 * num10;
					double num23 = num9 * 1.5 - num10 * 0.5;
					num11 = num8 * num11 + num7 * num23;
					num20 = num7 * num11 + num8 * num20;
					double num24 = num11 * 1.5 - num20 * 0.5;
					num12 = num8 * num12 + num7 * num24;
					num13 = num7 * num12 + num8 * num13;
					num4 = num12 * 1.5 - num13 * 0.5;
					num14 = num8 * num14 + num7 * Math.Abs(num22);
					num15 = num7 * num14 + num8 * num15;
					double num25 = num14 * 1.5 - num15 * 0.5;
					num16 = num8 * num16 + num7 * num25;
					num17 = num7 * num16 + num8 * num17;
					double num26 = num16 * 1.5 - num17 * 0.5;
					num18 = num8 * num18 + num7 * num26;
					num19 = num7 * num18 + num8 * num19;
					num5 = num18 * 1.5 - num19 * 0.5;
					if (num >= num2 && num6 != num21)
					{
						num3 = 1.0;
					}
					if (num == num2 && num3 == 0.0)
					{
						num2 = 0.0;
					}
				}
				double num27;
				if (num < num2 && num5 > 1E-10)
				{
					num27 = (num4 / num5 + 1.0) * 50.0;
					if (num27 > 100.0)
					{
						num27 = 100.0;
					}
					if (num27 < 0.0)
					{
						num27 = 0.0;
					}
				}
				else
				{
					num27 = 50.0;
				}
				array[i] = num27;
			}
			return array;
		}

		// Token: 0x170002BF RID: 703
		public IContext Context
		{
			// Token: 0x06000897 RID: 2199 RVA: 0x00024430 File Offset: 0x00022630
			get;
			// Token: 0x06000898 RID: 2200 RVA: 0x00024438 File Offset: 0x00022638
			set;
		}

		// Token: 0x170002BE RID: 702
		[HandlerParameter(true, "14", Min = "0", Max = "60", Step = "1")]
		public int length
		{
			// Token: 0x06000893 RID: 2195 RVA: 0x00023F9A File Offset: 0x0002219A
			get;
			// Token: 0x06000894 RID: 2196 RVA: 0x00023FA2 File Offset: 0x000221A2
			set;
		}
	}
}
